use std::io::{self, Write};

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
    Array(Vec<Value>),
    JSON(serde_json::Value)
}

impl Value {
    pub fn serialize<W>(&self, writer: &mut W) -> Result<(), io::Error>
    where
        W: Write,
    {
        match self {
            Value::Bool(b) => writer.write_all(&(*b as u8).to_ne_bytes()),
            Value::Int {
                value,
                endian,
                width,
            } => match (endian, width) {
                (_, IntWidth::Eight) => writer.write_all(&(*value as i8).to_ne_bytes()),
                (Endian::LittleEndian, IntWidth::Sixteen) => {
                    writer.write_all(&(*value as i16).to_le_bytes())
                }
                (Endian::LittleEndian, IntWidth::Thirtytwo) => {
                    writer.write_all(&(*value as i32).to_le_bytes())
                }
                (Endian::LittleEndian, IntWidth::Sixtyfour) => {
                    writer.write_all(&value.to_le_bytes())
                }
                (Endian::BigEndian, IntWidth::Sixteen) => {
                    writer.write_all(&(*value as i16).to_be_bytes())
                }
                (Endian::BigEndian, IntWidth::Thirtytwo) => {
                    writer.write_all(&(*value as i32).to_be_bytes())
                }
                (Endian::BigEndian, IntWidth::Sixtyfour) => writer.write_all(&value.to_be_bytes()),
            },
            Value::UInt {
                value,
                endian,
                width,
            } => match (endian, width) {
                (_, IntWidth::Eight) => writer.write_all(&(*value as u8).to_ne_bytes()),
                (Endian::LittleEndian, IntWidth::Sixteen) => {
                    writer.write_all(&(*value as u16).to_le_bytes())
                }
                (Endian::LittleEndian, IntWidth::Thirtytwo) => {
                    writer.write_all(&(*value as u32).to_le_bytes())
                }
                (Endian::LittleEndian, IntWidth::Sixtyfour) => {
                    writer.write_all(&value.to_le_bytes())
                }
                (Endian::BigEndian, IntWidth::Sixteen) => {
                    writer.write_all(&(*value as u16).to_be_bytes())
                }
                (Endian::BigEndian, IntWidth::Thirtytwo) => {
                    writer.write_all(&(*value as u32).to_be_bytes())
                }
                (Endian::BigEndian, IntWidth::Sixtyfour) => writer.write_all(&value.to_be_bytes()),
            },
            Value::Float {
                value,
                endian,
                width,
            } => match (endian, width) {
                (Endian::LittleEndian, FloatWidth::Thirtytwo) => {
                    writer.write_all(&(*value as f32).to_le_bytes())
                }
                (Endian::LittleEndian, FloatWidth::Sixtyfour) => {
                    writer.write_all(&value.to_le_bytes())
                }
                (Endian::BigEndian, FloatWidth::Thirtytwo) => {
                    writer.write_all(&(*value as f32).to_be_bytes())
                }
                (Endian::BigEndian, FloatWidth::Sixtyfour) => {
                    writer.write_all(&value.to_be_bytes())
                }
            },
            Value::String { value, encoding } => encoding.encode(value, writer),
            Value::Array(array) => {
                for value in array {
                    value.serialize(writer)?;
                }
                Ok(())
            }
            Value::JSON(value) => {
                serde_json::to_writer(writer, value)?;
                Ok(())
            }
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
    fn encode<W>(&self, value: &str, mut writer: W) -> Result<(), io::Error>
    where
        W: Write,
    {
        match self {
            StringEncoding::UTF8 => writer.write_all(value.as_bytes()),
            StringEncoding::UTF16BE => {
                writer.write_all(&0xFEFFu16.to_be_bytes())?;
                for c in value.encode_utf16().map(u16::to_be_bytes) {
                    writer.write_all(&c)?;
                }
                Ok(())
            }
            StringEncoding::UTF16LE => {
                writer.write_all(&0xFEFFu16.to_le_bytes())?;
                for c in value.encode_utf16().map(u16::to_le_bytes) {
                    writer.write_all(&c)?;
                }
                Ok(())
            }
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
