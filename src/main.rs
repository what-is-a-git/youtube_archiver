use std::{fs::File, io::Write};

use serde::{Serialize, Deserialize};

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

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            request_video(DownloadParameters {
                url: String::from("https://youtu.be/agTY4O0qsrc"),
                video_codec: String::from("h264"),
                filename: String::from("source_h264.mp4"),
            }).await;
        });
}

async fn request_video(params: DownloadParameters) {
    let result = download_video(params).await;

    if result.is_ok() {
        println!("Successfully downloaded requested video!");
    } else {
        let error = result.err().unwrap();
        println!("Error while downloading video!\nError: {error}");
    }
}

struct DownloadParameters {
    url: String,
    video_codec: String,
    filename: String,
}

async fn download_video(params: DownloadParameters) -> Result<(), String> {
    let client = reqwest::Client::new();
    let result = client.post("https://api.cobalt.tools/api/json")
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&RequestBody {
            url: params.url.as_str(),
            vCodec: params.video_codec.as_str(),
            vQuality: "max",
            aFormat: "best",
            filenamePattern: "pretty",
            isAudioOnly: false,
            isTTFullAudio: false,
            isAudioMuted: false,
            dubLang: false,
            disableMetadata: false,
            twitterGif: false,
            tiktokH265: false,
        })
        .send().await;

    if result.is_ok() {
        println!("Got a successful response from api.cobalt.tools/api/json!");

        let response = result.unwrap().json::<ResponseBody>().await.unwrap();
        let status = response.status;
        match status.as_str() {
            "error" => {
                let text = response.text.unwrap();
                println!("Got an error posting to the api!\nMessage: {text}");
                return Err(text);
            },
            "rate-limit" => {
                println!("Reached the rate-limit on cobalt's api! Maybe try again later.");
                return Err(String::from("Rate limited."));
            },
            "stream" => {
                println!("Got a valid video stream! Getting file now.");
                let get_url = response.url.unwrap();
                let get_response = client.get(get_url)
                    .header("Accept", "application/json")
                    .header("Content-Type", "application/json")
                    .send().await.unwrap();
                let video_contents = get_response.bytes().await.unwrap();
                let mut output_file = File::create(params.filename.clone()).unwrap();
                let write_result = output_file.write_all(&video_contents);
                
                if write_result.is_ok() {
                    println!("Wrote requested to file successfully at {}!", params.filename);
                } else {
                    println!("Couldn't write to file {}... Error: {:?}", params.filename, write_result.err());
                }
            },
            _ => {
                println!("Unimplemented status recieved!\nStatus: {status}");
                return Err(String::from("Unimplemented status."));
            },
        }
    } else {
        println!("Got an error while posting to api.cobalt.tools/api/json! Maybe check your internet connection?");
        return Err(String::from("Error posting to api."));
    }

    Ok(())
}
