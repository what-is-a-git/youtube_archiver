// crates //
use clap::{ArgAction, Parser};
use std::fs::create_dir_all;

// modules //
mod http;
mod log;
use log::*;
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

    #[arg(short, long, default_value_t = true, action = ArgAction::Set)]
    video: bool,

    #[arg(short, long, default_value_t = true, action = ArgAction::Set)]
    metadata: bool,

    #[arg(short, long, default_value_t = true, action = ArgAction::Set)]
    streams_and_premieres: bool,

    #[arg(short, long, default_value_t = String::from("https://yt.lemnoslife.com"))]
    api: String,
}

fn main() {
    let args = Arguments::parse();
    create_dir_all(&args.dir).unwrap();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            if args.url.contains(",") {
                let videos_raw: Vec<&str> = args.url.split(",").collect();
                let mut videos: Vec<String> = Vec::new();
                for raw in videos_raw {
                    videos.push(String::from(raw));
                }

                get_videos(&args, videos).await;
            } else {
                if args.url.contains("@") {
                    get_channel(&args).await;
                } else {
                    get_video(&args).await;
                }
            }
        });
}

async fn get_channel(args: &Arguments) {
    let videos = request_channel(ChannelRequest {
        url: &args.url,
        api: args.api.clone(),
        include_streams_and_premieres: args.streams_and_premieres,
    })
    .await;
    if videos.is_err() {
        let error = videos.err().unwrap();
        failure(format!(
            "Encountered an error while getting channel videos! Error: {error}"
        ));
        return;
    }

    for video in videos.unwrap() {
        let passed_args = Arguments {
            url: video.clone(),
            dir: format!("{}/{}", args.dir, get_id_from_url(&video)),
            video: args.video,
            metadata: args.metadata,
            streams_and_premieres: args.streams_and_premieres,
            api: args.api.clone(),
        };
        get_video(&passed_args).await;
    }

    success(String::from("Finished downloading all videos from provided channel! Check for any potential errors in the console just in case."));
}

async fn get_videos(args: &Arguments, videos: Vec<String>) {
    request(format!("Downloading all videos from list {:?}", &videos));
    for video in videos {
        let passed_args = Arguments {
            url: video.clone(),
            dir: format!("{}/{}", args.dir, get_id_from_url(&video)),
            video: args.video,
            metadata: args.metadata,
            streams_and_premieres: args.streams_and_premieres,
            api: args.api.clone(),
        };
        get_video(&passed_args).await;
    }

    success(String::from("Finished downloading all videos from provided channel! Check for any potential errors in the console just in case."));
}

async fn get_video(args: &Arguments) {
    if args.metadata {
        request_metadata(
            MetadataParameters {
                url: &args.url,
                dir: &args.dir,
            },
            args.api.clone(),
        )
        .await;
    }

    if args.video {
        request_video(VideoParameters {
            url: &args.url,
            video_codec: String::from("h264"),
            filename: format!("{}/source_h264.mp4", &args.dir),
        })
        .await;
    }
}
