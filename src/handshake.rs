use rand::Rng;

#[derive(Debug)]
pub struct Handshake {
    pub length: u8,
    pub protocol: String,
    pub reserved: u64,
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20]) -> Self {
        let mut rng = rand::rng();
        let peer_id: [u8; 20] = rng.random();

        Handshake {
            length: 19,
            protocol: String::from("BitTorrent protocol"),
            reserved: 0,
            info_hash,
            peer_id,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.length);
        bytes.extend_from_slice(self.protocol.as_bytes());
        bytes.extend_from_slice(&self.reserved.to_be_bytes());
        bytes.extend_from_slice(&self.info_hash);
        bytes.extend_from_slice(&self.peer_id);
        bytes
    }

    pub fn from_bytes(buffer: &[u8]) -> Self {
        if buffer.len() < 49 {
            panic!("expected 49 bytes long");
        }

        let length = buffer[0];

        let protocol = match std::str::from_utf8(&buffer[1..20]) {
            Ok(s) => s.to_string(),
            Err(_) => panic!("expected 'BitTorrent protocol' string"),
        };

        let reserved = u64::from_be_bytes(buffer[20..28].try_into().unwrap());

        let info_hash: [u8; 20] = buffer[28..48].try_into().unwrap();

        let peer_id: [u8; 20] = buffer[48..68].try_into().unwrap();

        Handshake {
            length,
            protocol,
            reserved,
            info_hash,
            peer_id,
        }
    }
}
