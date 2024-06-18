use std::{fs::{create_dir_all, File}, io::Write};
use serde::{Serialize, Deserialize};
use colored::*;
use clap::Parser;

#[allow(non_snake_case)] // needed for cobalt api
#[derive(Serialize)]
struct RequestBody<'a> {
    url: &'a str,
    vCodec: &'a str,
    vQuality: &'a str,
    aFormat: &'a str,
    filenamePattern: &'a str,
    isAudioOnly: bool,
    isTTFullAudio: bool,
    isAudioMuted: bool,
    dubLang: bool,
    disableMetadata: bool,
    twitterGif: bool,
    tiktokH265: bool,
}

#[allow(non_snake_case)] // needed for cobalt api
#[derive(Deserialize)]
struct ResponseBody {
    status: String,
    text: Option<String>,
    url: Option<String>,
    // pickerType: Option<String>,
    // picker: some array type lol
    // audio: Option<String>,
}

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

#[derive(Debug)]
struct MetadataParameters<'a> {
    url: &'a String,
    dir: &'a String,
}

async fn request_metadata(params: MetadataParameters<'_>) {
    let id = get_id_from_url(params.url);
    let meta_result = download_metadata(format!("https://yt.lemnoslife.com/noKey/videos?part=snippet&id={id}")).await;

    if meta_result.is_err() {
        let error = meta_result.err().unwrap();
        println!("{} {}", " FAILURE ".on_red(), error);
        return;
    }

    let meta = meta_result.unwrap();
    assert!(meta.items.len() > 0, "YouTube API Response had no metadata items! Maybe try again later?");

    let write_result = write_metadata(&meta.items[0], params.dir);
    if write_result.is_err() {
        let error = write_result.err().unwrap();
        println!("{} {}", " FAILURE ".on_red(), error);
        return;
    }

    let thumbnails = &meta.items[0].snippet.thumbnails;
    if thumbnails.default.is_some() {
        request_thumbnail(ThumbnailParameters {
            url: &thumbnails.default.as_ref().unwrap().url,
            filename: format!("{}/thumb_default.jpg", params.dir),
        }).await;
    }

    if thumbnails.medium.is_some() {
        request_thumbnail(ThumbnailParameters {
            url: &thumbnails.medium.as_ref().unwrap().url,
            filename: format!("{}/thumb_medium.jpg", params.dir),
        }).await;
    }
    
    if thumbnails.high.is_some() {
        request_thumbnail(ThumbnailParameters {
            url: &thumbnails.high.as_ref().unwrap().url,
            filename: format!("{}/thumb_high.jpg", params.dir),
        }).await;
    }

    if thumbnails.standard.is_some() {
        request_thumbnail(ThumbnailParameters {
            url: &thumbnails.standard.as_ref().unwrap().url,
            filename: format!("{}/thumb_standard.jpg", params.dir),
        }).await;
    }

    if thumbnails.maxres.is_some() {
        request_thumbnail(ThumbnailParameters {
            url: &thumbnails.maxres.as_ref().unwrap().url,
            filename: format!("{}/thumb_maxres.jpg", params.dir),
        }).await;
    }
}

#[derive(Serialize)]
struct ArchivedMetadata {
    id: String,
    title: String,
    description: String,
    creator: String,
    publish_date: String,
    tags: Vec<String>,
}

fn write_metadata(input: &ItemResponse, dir: &String) -> Result<(), String> {
    let mut tags: Vec<String> = Vec::new();

    if input.snippet.tags.is_some() {
        tags = input.snippet.tags.as_ref().unwrap().clone();
    }

    let output_data = ArchivedMetadata {
        title: input.snippet.title.clone(),
        description: input.snippet.description.clone(),
        creator: input.snippet.channelTitle.clone(),
        publish_date: input.snippet.publishedAt.clone(),
        tags: tags,
        id: input.id.clone(),
    };
    let output_filename = format!("{dir}/meta.json");
    let mut output_file = File::create(&output_filename).unwrap();
    let output_contents = serde_json::to_string_pretty(&output_data).unwrap();
    let write_result = output_file.write_all(output_contents.as_bytes());
    
    if !write_result.is_ok() {
        return Err(format!("Couldn't write to file {}... Error: {:?}", &output_filename, write_result.err()));
    }

    println!("{} Wrote to requested file {} successfully!", " SUCCESS ".on_green(), &output_filename);
    Ok(())
}

#[allow(non_snake_case)] // needed for youtube api
#[derive(Debug, Deserialize)]
struct ThumbnailResponse {
    url: String,
}

#[allow(non_snake_case)] // needed for youtube api
#[derive(Debug, Deserialize)]
struct ThumbnailsResponse {
    default: Option<ThumbnailResponse>,
    medium: Option<ThumbnailResponse>,
    high: Option<ThumbnailResponse>,
    standard: Option<ThumbnailResponse>,
    maxres: Option<ThumbnailResponse>,
}

#[allow(non_snake_case)] // needed for youtube api
#[derive(Debug, Deserialize)]
struct SnippetResponse {
    publishedAt: String,
    title: String,
    description: String,
    thumbnails: ThumbnailsResponse,
    channelTitle: String,
    tags: Option<Vec<String>>,
}

#[allow(non_snake_case)] // needed for youtube api
#[derive(Debug, Deserialize)]
struct ItemResponse {
    id: String,
    snippet: SnippetResponse,
}

#[allow(non_snake_case)] // needed for youtube api
#[derive(Debug, Deserialize)]
struct YouTubeResponse {
    items: Vec<ItemResponse>,
}

async fn download_metadata(url: String) -> Result<YouTubeResponse, String> {
    println!("{} Requesting metadata at this url: {}", " REQUEST ".on_cyan(), &url);
    let client = reqwest::Client::new();
    let result = client.get(url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send().await;

    if result.is_err() {
        let error = result.err().unwrap();
        return Err(format!("Error while getting metadata! Error: {error}"));
    }

    let contents = result.unwrap().text().await.unwrap();
    let bytes = contents.as_bytes();
    let response = serde_json::from_slice(&bytes);
    if response.is_err() {
        let error = response.err().unwrap();
        return Err(format!("Error while parsing metadata! Error: {error}, Original Data: {contents}"));
    }

    Ok(response.unwrap())
}

#[derive(Debug)]
struct ThumbnailParameters<'a> {
    url: &'a String,
    filename: String,
}

async fn request_thumbnail(params: ThumbnailParameters<'_>) {
    println!("{} Downloading thumbnail with these parameters: {params:?}", " REQUEST ".on_cyan());
    let result = download_thumbnail(params).await;

    if result.is_err() {
        let error = result.err().unwrap();
        println!("{} Error while downloading thumbnail! Error: {error}", " FAILURE ".on_red());
        return;
    }
}

async fn download_thumbnail(params: ThumbnailParameters<'_>) -> Result<(), String> {
    let client = reqwest::Client::new();
    let result = client.get(params.url)
        .send().await;

    if result.is_err() {
        return Err(format!("Failed to request {}! Error: {:?}", params.url, result.err().unwrap()));
    }

    let response = result.unwrap();
    let contents = response.bytes().await.unwrap();
    let mut output_file = File::create(params.filename.clone()).unwrap();
    let write_result = output_file.write_all(&contents);
    
    if !write_result.is_ok() {
        return Err(format!("Couldn't write to file {}... Error: {:?}", params.filename, write_result.err()));
    }

    println!("{} Wrote to requested file {} successfully!", " SUCCESS ".on_green(), params.filename);
    Ok(())
}

fn get_id_from_url(url: &String) -> String {
    assert!(url.find("youtu").is_some(), "Make sure to provide a valid YouTube URL!");
    let clean_url = url.split_at(url.find("youtu").unwrap()).1;

    if clean_url.starts_with("youtube.com/watch?v=") { // youtube.com/watch?v=id
        String::from(clean_url.split_at(clean_url.find("=").unwrap() + 1).1)
    } else { // youtu.be/id
        String::from(clean_url.split_at(clean_url.find("/").unwrap() + 1).1)
    }
}

#[derive(Debug)]
struct VideoParameters<'a> {
    url: &'a String,
    video_codec: String,
    filename: String,
}

async fn request_video(params: VideoParameters<'_>) {
    println!("{} Downloading video with these parameters: {params:?}", " REQUEST ".on_cyan());
    let result = download_video(params).await;

    if result.is_err() {
        let error = result.err().unwrap();
        println!("{} Error while downloading video! Error: {error}", " FAILURE ".on_red());
        return;
    }
}

async fn download_video(params: VideoParameters<'_>) -> Result<(), String> {
    let client = reqwest::Client::new();
    let initial_result = client.post("https://api.cobalt.tools/api/json")
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&RequestBody {
            url: params.url.as_str(),
            vCodec: params.video_codec.as_str(),
            vQuality: "max",
            aFormat: "best",
            filenamePattern: "classic",
            isAudioOnly: false,
            isTTFullAudio: false,
            isAudioMuted: false,
            dubLang: false,
            disableMetadata: false,
            twitterGif: false,
            tiktokH265: false,
        })
        .send().await;

    if !initial_result.is_ok() {
        return Err(String::from("Got an error while posting to api.cobalt.tools/api/json! Maybe check your internet connection?"));
    }

    println!("{} Got response from the cobalt api!", " SUCCESS ".on_green());

    let initial_response = initial_result.unwrap().json::<ResponseBody>().await.unwrap();
    let status = initial_response.status;
    match status.as_str() {
        "error" => {
            let text = initial_response.text.unwrap();
            return Err(format!("Got an error posting to the cobalt api! Message: {text}"));
        },
        "rate-limit" => {
            return Err(String::from("Rate-limited from the cobalt api."));
        },
        "stream" => {
            println!("{} Got a valid video stream! Now getting file.", " SUCCESS ".on_green());
            let get_url = initial_response.url.unwrap();
            let get_response = client.get(get_url)
                .header("Accept", "application/json")
                .header("Content-Type", "application/json")
                .send().await.unwrap();
            let video_contents = get_response.bytes().await.unwrap();
            let mut output_file = File::create(params.filename.clone()).unwrap();
            let write_result = output_file.write_all(&video_contents);
            
            if !write_result.is_ok() {
                return Err(format!("Couldn't write to file {}. Error: {:?}", params.filename, write_result.err().unwrap()));
            }

            println!("{} Wrote to requested file {} successfully!", " SUCCESS ".on_green(), params.filename);
        },
        _ => {
            return Err(format!("No implementation for status {status}."));
        },
    }

    Ok(())
}
