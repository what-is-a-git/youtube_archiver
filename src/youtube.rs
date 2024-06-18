use std::{fs::File, io::Write};
use serde::{Serialize, Deserialize};
use crate::log::*;

#[derive(Debug)]
pub struct MetadataParameters<'a> {
    pub url: &'a String,
    pub dir: &'a String,
}

pub async fn request_metadata(params: MetadataParameters<'_>) {
    let id = get_id_from_url(params.url);
    let meta_result = download_metadata(format!("https://yt.lemnoslife.com/noKey/videos?part=snippet&id={id}")).await;

    if meta_result.is_err() {
        let error = meta_result.err().unwrap();
        failure(error);
        return;
    }

    let meta = meta_result.unwrap();
    assert!(meta.items.len() > 0, "YouTube API Response had no metadata items! Maybe try again later?");

    let write_result = write_metadata(&meta.items[0], params.dir);
    if write_result.is_err() {
        let error = write_result.err().unwrap();
        failure(error);
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

    success(format!("Wrote to requested file {} successfully!", &output_filename));
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
    request(format!("Requesting metadata at this url: {}", &url));
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
    request(format!("Downloading thumbnail with these parameters: {params:?}"));
    let result = download_thumbnail(params).await;

    if result.is_err() {
        let error = result.err().unwrap();
        failure(format!("Error while downloading thumbnail! Error: {error}"));
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

    success(format!("Wrote to requested file {} successfully!", params.filename));
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