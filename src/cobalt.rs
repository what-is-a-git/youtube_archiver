use std::{fs::File, io::Write};
use serde::{Serialize, Deserialize};
use crate::log::*;

#[allow(non_snake_case)] // needed for cobalt api
#[derive(Serialize)]
pub struct RequestBody<'a> {
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
pub struct ResponseBody {
    status: String,
    text: Option<String>,
    url: Option<String>,
}

#[derive(Debug)]
pub struct VideoParameters<'a> {
    pub url: &'a String,
    pub video_codec: String,
    pub filename: String,
}

pub async fn request_video(params: VideoParameters<'_>) {
    request(format!("Downloading video with these parameters: {params:?}"));
    let result = download_video(params).await;

    if result.is_err() {
        let error = result.err().unwrap();
        failure(format!("Error while downloading video! Error: {error}"));
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

    success(String::from("Got response from the cobalt api!"));

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
            success(String::from("Got a valid video stream! Now getting file."));
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

            success(format!("Wrote to requested file {} successfully!", params.filename));
        },
        _ => {
            return Err(format!("No implementation for status {status}."));
        },
    }

    Ok(())
}