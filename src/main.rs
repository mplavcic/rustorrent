mod bencode;
mod metainfo;

use bencode::decode_bencoded_value;
use clap::Parser;
use clap::Subcommand;
use metainfo::MetaInfo;
use sha1::Digest;
use sha1::Sha1;
use std::fs;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Decode { encoded_value: String },
    Info { file_path: String },
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
            let metainfo: MetaInfo = serde_bencode::from_bytes(&bencoded_metainfo).unwrap();

            // TODO: multiple files
            println!("Tracker URL: {}", metainfo.tracker_url);
            println!("Length: {}", metainfo.info.piece_length);

            let mut hasher = Sha1::new();
            let bencoded_metainfo_info = serde_bencode::to_bytes(&metainfo.info).unwrap();
            hasher.update(&bencoded_metainfo_info);
            let bencoded_metainfo_info_hash = hasher.finalize();

            println!("Info Hash: {}", hex::encode(&bencoded_metainfo_info_hash));

            println!("Piece Length: {}", metainfo.info.piece_length);

            println!("Piece Hashes:");
            for hash in metainfo.info.pieces.0 {
                println!("{}", hex::encode(hash));
            }
        }
    }
}
