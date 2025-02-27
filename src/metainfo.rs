use serde::de;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use sha1::Digest;
use sha1::Sha1;
use std::fmt;

#[derive(Debug, Clone, Deserialize)]
pub struct Metainfo {
    #[serde(rename = "announce")]
    pub tracker_url: String,
    pub info: Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    pub pieces: Hashes,
    #[serde(flatten)]
    pub keys: Keys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Keys {
    SingleFile { length: usize },
    MultiFile { files: Vec<File> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub length: usize,
    pub path: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Hashes(pub Vec<[u8; 20]>);
struct HashesVisitor;

impl Metainfo {
    pub fn info_hash(&self) -> [u8; 20] {
        let mut hasher = Sha1::new();
        let bencoded_metainfo_info = serde_bencode::to_bytes(&self.info).unwrap();
        hasher.update(&bencoded_metainfo_info);
        let bencoded_metainfo_info_hash = hasher.finalize();
        bencoded_metainfo_info_hash.into()
    }
}

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

impl Serialize for Hashes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let single_slice = self.0.concat();
        serializer.serialize_bytes(&single_slice)
    }
}
