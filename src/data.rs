use std::iter;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Data {
    topic: String,
    data: Value,
}

impl Data {
    /// Get a reference to the data's data.
    pub fn data(&self) -> &Value {
        &self.data
    }

    /// Get a reference to the data's topic.
    pub fn topic(&self) -> &str {
        &self.topic
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    UInt {
        value: u64,
        #[serde(default)]
        endian: Endian,
        #[serde(default)]
        width: IntWidth,
    },
    Int {
        value: i64,
        #[serde(default)]
        endian: Endian,
        #[serde(default)]
        width: IntWidth,
    },
    Float {
        value: f64,
        #[serde(default)]
        endian: Endian,
        #[serde(default)]
        width: FloatWidth,
    },
    String {
        value: String,
        #[serde(default)]
        encoding: StringEncoding,
    },
}

impl Value {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Value::Bool(b) => (*b as u8).to_be_bytes().to_vec(),
            Value::Int {
                value,
                endian,
                width,
            } => match (endian, width) {
                (_, IntWidth::Eight) => (*value as i8).to_ne_bytes().to_vec(),
                (Endian::LittleEndian, IntWidth::Sixteen) => (*value as i16).to_le_bytes().to_vec(),
                (Endian::LittleEndian, IntWidth::Thirtytwo) => {
                    (*value as i32).to_le_bytes().to_vec()
                }
                (Endian::LittleEndian, IntWidth::Sixtyfour) => value.to_le_bytes().to_vec(),
                (Endian::BigEndian, IntWidth::Sixteen) => (*value as i16).to_be_bytes().to_vec(),
                (Endian::BigEndian, IntWidth::Thirtytwo) => (*value as i32).to_be_bytes().to_vec(),
                (Endian::BigEndian, IntWidth::Sixtyfour) => value.to_be_bytes().to_vec(),
            },
            Value::UInt {
                value,
                endian,
                width,
            } => match (endian, width) {
                (_, IntWidth::Eight) => (*value as u8).to_ne_bytes().to_vec(),
                (Endian::LittleEndian, IntWidth::Sixteen) => (*value as u16).to_le_bytes().to_vec(),
                (Endian::LittleEndian, IntWidth::Thirtytwo) => {
                    (*value as u32).to_le_bytes().to_vec()
                }
                (Endian::LittleEndian, IntWidth::Sixtyfour) => value.to_le_bytes().to_vec(),
                (Endian::BigEndian, IntWidth::Sixteen) => (*value as u16).to_be_bytes().to_vec(),
                (Endian::BigEndian, IntWidth::Thirtytwo) => (*value as u32).to_be_bytes().to_vec(),
                (Endian::BigEndian, IntWidth::Sixtyfour) => value.to_be_bytes().to_vec(),
            },
            Value::Float {
                value,
                endian,
                width,
            } => match (endian, width) {
                (Endian::LittleEndian, FloatWidth::Thirtytwo) => {
                    (*value as f32).to_le_bytes().to_vec()
                }
                (Endian::LittleEndian, FloatWidth::Sixtyfour) => value.to_le_bytes().to_vec(),
                (Endian::BigEndian, FloatWidth::Thirtytwo) => {
                    (*value as f32).to_be_bytes().to_vec()
                }
                (Endian::BigEndian, FloatWidth::Sixtyfour) => value.to_be_bytes().to_vec(),
            },
            Value::String { value, encoding } => encoding.encode(value),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum StringEncoding {
    UTF8,
    UTF16BE,
    UTF16LE,
}

impl StringEncoding {
    fn encode(&self, value: &str) -> Vec<u8> {
        match self {
            StringEncoding::UTF8 => value.as_bytes().to_vec(),
            StringEncoding::UTF16BE => iter::once(0xFEFFu16)
                .chain(value.encode_utf16())
                .map(|c| c.to_be_bytes())
                .fold(Vec::new(), |mut acc, val| {
                    acc.extend_from_slice(&val);
                    acc
                }),
            StringEncoding::UTF16LE => iter::once(0xFEFFu16)
                .chain(value.encode_utf16())
                .map(|c| c.to_le_bytes())
                .fold(Vec::new(), |mut acc, val| {
                    acc.extend_from_slice(&val);
                    acc
                }),
        }
    }
}

impl Default for StringEncoding {
    fn default() -> Self {
        StringEncoding::UTF8
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum Endian {
    LittleEndian,
    BigEndian,
}

impl Default for Endian {
    fn default() -> Self {
        Endian::BigEndian
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum IntWidth {
    #[serde(alias = "8")]
    Eight,
    #[serde(alias = "16")]
    Sixteen,
    #[serde(alias = "32")]
    Thirtytwo,
    #[serde(alias = "64")]
    Sixtyfour,
}

impl Default for IntWidth {
    fn default() -> Self {
        IntWidth::Sixtyfour
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum FloatWidth {
    #[serde(alias = "32")]
    Thirtytwo,
    #[serde(alias = "64")]
    Sixtyfour,
}

impl Default for FloatWidth {
    fn default() -> Self {
        FloatWidth::Sixtyfour
    }
}
