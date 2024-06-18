// crates //
use std::fs::create_dir_all;//, File}, io::Write};
// use serde::{Serialize, Deserialize};
// use colored::*;
use clap::Parser;

// modules //
mod log;
mod cobalt;
use cobalt::*;
mod youtube;
use youtube::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[arg(short, long)]
    url: String,

    #[arg(short, long)]
    dir: String,

    #[arg(short, long, default_value_t = true)]
    video: bool,

    #[arg(short, long, default_value_t = true)]
    metadata: bool,
}

fn main() {
    let args = Arguments::parse();
    let directory = args.dir;
    let url = args.url;

    create_dir_all(&directory).unwrap();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            if args.metadata {
                request_metadata(MetadataParameters {
                    url: &url,
                    dir: &directory,
                }).await;
            }

            if args.video {
                request_video(VideoParameters {
                    url: &url,
                    video_codec: String::from("h264"),
                    filename: format!("{}/source_h264.mp4", &directory),
                }).await;
            }
        });
}
