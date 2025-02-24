use serde::de;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct MetaInfo {
    #[serde(rename = "announce")]
    pub tracker_url: String,
    pub info: Info,
}

#[derive(Debug, Deserialize)]
pub struct Info {
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    pub pieces: Hashes,
    #[serde(flatten)]
    pub keys: Keys,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Keys {
    SingleFile { length: usize },
    MultiFile { files: Vec<File> },
}

#[derive(Debug, Deserialize)]
pub struct File {
    pub length: usize,
    pub path: Vec<String>,
}

#[derive(Debug)]
struct Hashes(Vec<[u8; 20]>);
struct HashesVisitor;

impl<'de> Visitor<'de> for HashesVisitor {
    type Value = Hashes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("list of bytes representing file hashes")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value.len() % 20 != 0 {
            return Err(E::custom("byte slice length is not a multiple of 20"));
        }

        Ok(Hashes(
            value
                .chunks_exact(20)
                .map(|slice| slice.try_into().expect("guaranteed to be length 20"))
                .collect(),
        ))
    }
}

impl<'de> Deserialize<'de> for Hashes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(HashesVisitor)
    }
}
