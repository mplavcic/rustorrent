mod bencode;
mod handshake;
mod metainfo;
mod tracker;

use std::io::{Read, Write};
use bencode::decode_bencoded_value;
use clap::Parser;
use clap::Subcommand;
use handshake::Handshake;
use metainfo::Metainfo;
use std::fs;
use tracker::TrackerResponse;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Decode {
        encoded_value: String,
    },
    Info {
        file_path: String,
    },
    Peers {
        file_path: String,
    },
    Handshake {
        file_path: String,
        peer: std::net::SocketAddrV4,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Decode { encoded_value } => {
            let decoded_value = decode_bencoded_value(&encoded_value);
            println!("{}", decoded_value.to_string());
        }
        Commands::Info { file_path } => {
            let bencoded_metainfo = fs::read(file_path).unwrap();
            let metainfo: Metainfo = serde_bencode::from_bytes(&bencoded_metainfo).unwrap();

            // TODO: multiple files
            println!("Tracker URL: {}", metainfo.tracker_url);
            println!("Length: {}", metainfo.info.piece_length);

            let bencoded_metainfo_info_hash = metainfo.info_hash();
            println!("Info Hash: {}", hex::encode(&bencoded_metainfo_info_hash));

            println!("Piece Length: {}", metainfo.info.piece_length);

            println!("Piece Hashes:");
            for hash in metainfo.info.pieces.0 {
                println!("{}", hex::encode(hash));
            }
        }
        Commands::Peers { file_path } => {
            let bencoded_metainfo = fs::read(file_path).unwrap();
            let metainfo: Metainfo = serde_bencode::from_bytes(&bencoded_metainfo).unwrap();

            let tracker_url = &metainfo.tracker_url;

            let info_hash = metainfo.info_hash();
            let urlencoded_info_hash =
                url::form_urlencoded::byte_serialize(&info_hash).collect::<String>();

            let length = if let metainfo::Keys::SingleFile { length } = metainfo.info.keys {
                length
            } else {
                //TODO: multiple files
                todo!();
            };

            let url = format!(
                "{tracker_url}/?info_hash={urlencoded_info_hash}&peer_id=00112233445566778899&port=6881&uploaded=0&downloaded=0&left={length}&compact=1"
                );

            let response_bytes = reqwest::blocking::get(url).unwrap().bytes().unwrap();
            let response: TrackerResponse = serde_bencode::from_bytes(&response_bytes).expect("yo");

            for peer in response.peers.0 {
                println!("{peer}");
            }
        }
        Commands::Handshake { file_path, peer } => {
            let mut stream =
                std::net::TcpStream::connect(format!("{peer}")).expect("jebena konekcija");
            let bencoded_metainfo = fs::read(file_path).unwrap();
            let metainfo: Metainfo = serde_bencode::from_bytes(&bencoded_metainfo).unwrap();

            let message = Handshake::new(metainfo.info_hash());

            stream
                .write_all(&message.to_bytes())
                .expect("streamulja write all");

            let mut buffer = [0; 68];
            stream.read(&mut buffer).expect("streamulj read");

            let handshake = Handshake::from_bytes(&buffer);

            println!("Peer ID: {}", hex::encode(handshake.peer_id));
        }
    }
}
