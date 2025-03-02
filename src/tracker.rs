use serde::de;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use std::fmt;

/*
#[derive(Debug, Clone, Deserialize)]
pub struct TrackerRequest {
    pub info_hash: [u8; 20],
    pub peer_id: usize,
    pub port: u16,
    pub uploaded: usize,
    pub downloaded: usize,
    pub left: usize,
    pub compact: u8,
}
*/

#[derive(Debug, Clone, Deserialize)]
pub struct TrackerResponse {
    pub interval: usize,
    pub peers: Peers,
}

#[derive(Debug, Clone)]
pub struct Peers(pub Vec<std::net::SocketAddrV4>);
pub struct PeersVisitor;

impl<'de> Visitor<'de> for PeersVisitor {
    type Value = Peers;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("list of bytes representing peers")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value.len() % 6 != 0 {
            return Err(E::custom("byte slice length is not a multiple of 6"));
        }

        Ok(Peers(
            value
                .chunks_exact(6)
                .map(|slice| {
                    std::net::SocketAddrV4::new(
                        std::net::Ipv4Addr::new(slice[0], slice[1], slice[2], slice[3]),
                        u16::from_be_bytes([slice[4], slice[5]]),
                    )
                })
                .collect(),
        ))
    }
}

impl<'de> Deserialize<'de> for Peers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PeersVisitor)
    }
}
