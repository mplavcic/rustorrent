mod bencode;
mod metainfo;

use bencode::decode_bencoded_value;
use clap::Parser;
use clap::Subcommand;
use metainfo::MetaInfo;
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
        }
    }
}
