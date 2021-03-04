mod data;

use std::time::{Duration, SystemTime};

use anyhow::{bail, Result};
use clap::{App, Arg};
use env_logger::Env;
use rumqttc::{EventLoop, MqttOptions, Publish, QoS, Request, Sender};
use tokio::{
    fs, select, task,
    time::{interval, Interval},
};
use tokio::{sync::watch, time::sleep};

use data::Data;

async fn data_watcher(path: String, tx: watch::Sender<Vec<Data>>) -> Result<()> {
    let mut interval = interval(Duration::from_millis(100));
    let mut modified = SystemTime::UNIX_EPOCH;
    loop {
        let meta = fs::metadata(&path).await?;
        let last_mod = meta.modified().unwrap();
        if modified < last_mod {
            let values = if let Ok(s) = fs::read_to_string(&path).await {
                s
            } else {
                continue;
            };
            match serde_json::from_str::<Vec<Data>>(&values) {
                Ok(vals) => {
                    log::info!("Replacing values with:\n{:#?}", vals);
                    tx.send(vals).map_err(|_| "").expect("Watchers died");
                    modified = last_mod;
                }
                Err(e) => {
                    log::debug!("Failed to read values: {:?}\n{}", e, values);
                }
            }
        }
        interval.tick().await;
    }
}

async fn sender(
    rx: watch::Receiver<Vec<Data>>,
    sink: Sender<Request>,
    mut interval: Interval,
) -> Result<()> {
    loop {
        let vals = rx.borrow().clone();
        for val in vals {
            let mut buf = Vec::new();
            val.data().serialize(&mut buf)?;
            let msg = Publish::new(val.topic(), QoS::AtLeastOnce, buf);
            sink.send(Request::Publish(msg))
                .await
                .expect("Eventloop rx seems to be dead.");
        }
        interval.tick().await;
    }
}

async fn eventloop_task(mut eventloop: EventLoop) -> Result<()> {
    loop {
        match eventloop.poll().await {
            Err(e) => {
                log::error!("Lost connection to MQTT Broker {:?}, retrying in 3s", e);
                sleep(Duration::from_secs(3)).await;
            }
            Ok(p) => {
                log::debug!("MQTT Event: {:?}", p)
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::new("mqtt-simulator")
        .arg(Arg::with_name("config"))
        .arg(
            Arg::with_name("host")
                .long("host")
                .short("h")
                .default_value("localhost"),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .short("p")
                .default_value("1883"),
        )
        .arg(
            Arg::with_name("client-id")
                .long("client-id")
                .short("i")
                .default_value("mqtt-simulator"),
        )
        .arg(
            Arg::with_name("send-interval")
                .long("send-interval")
                .short("t")
                .help("Send interval in milliseconds")
                .default_value("1000"),
        );
    let matches = app.get_matches();

    let path = matches.value_of("config").unwrap().to_string();
    let host = matches.value_of("host").unwrap();
    let port = matches.value_of("port").unwrap().parse()?;
    let send_interval = matches.value_of("send-interval").unwrap().parse()?;
    let client_id = matches.value_of("client-id").unwrap();

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    log::info!(
        "Sending data from {} to MQTT Broker at {}:{} as {}",
        path,
        host,
        port,
        client_id
    );
    let opts = MqttOptions::new(client_id, host, port);

    let eventloop = EventLoop::new(opts, 10);
    let requests_tx = eventloop.handle();
    let (data_tx, data_rx) = watch::channel(vec![]);

    let watcher = task::spawn(data_watcher(path, data_tx));

    let eventloop_task = task::spawn(eventloop_task(eventloop));

    let loop2 = task::spawn(sender(
        data_rx,
        requests_tx,
        interval(Duration::from_millis(send_interval)),
    ));
    select! {
        res = watcher => {
            bail!("Watcher died: {:?}", res)
        }
        res = loop2 => {
            bail!("Sender died: {:?}", res)
        },
        res = eventloop_task => {
            bail!("Eventloop died: {:?}", res)
        }
    };
}
