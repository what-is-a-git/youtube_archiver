use std::{fs::{create_dir_all, File}, io::Write};
use async_recursion::async_recursion;
use serde::{Serialize, Deserialize};
use crate::log::*;

#[derive(Debug)]
pub struct MetadataParameters<'a> {
    pub url: &'a String,
    pub dir: &'a String,
}

pub async fn request_metadata(params: MetadataParameters<'_>, api: String) {
    let id = get_id_from_url(params.url);
    let meta_result = download_metadata(format!("{api}/noKey/videos?part=snippet&id={id}")).await;

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
    create_dir_all(&dir).unwrap();
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

pub fn get_id_from_url(url: &String) -> String {
    assert!(url.find("youtu").is_some(), "Make sure to provide a valid YouTube URL!");
    let clean_url = url.split_at(url.find("youtu").unwrap()).1;

    if clean_url.starts_with("youtube.com/watch?v=") { // youtube.com/watch?v=id
        String::from(clean_url.split_at(clean_url.find("=").unwrap() + 1).1)
    } else { // youtu.be/id
        String::from(clean_url.split_at(clean_url.find("/").unwrap() + 1).1)
    }
}

#[allow(non_snake_case)] // needed for youtube api
#[derive(Deserialize)]
struct ChannelListResponse {
    items: Vec<ChannelResponse>,
}

#[allow(non_snake_case)] // needed for youtube api
#[derive(Deserialize)]
struct ChannelResponse {
    id: String,
}

pub async fn request_channel(url: &String, api: String, include_streams_and_premieres: bool) -> Result<Vec<String>, String> {
    let channel_handle = get_channel_handle_from_url(url);
    request(format!("Requesting channel ID from handle {}!", &channel_handle));
    let client = reqwest::Client::new();
    let id_url = format!("{api}/noKey/channels?part=id&forHandle=@{}", &channel_handle);
    let result = client.get(id_url)
        .send().await;
    if result.is_err() {
        let error = result.err().unwrap();
        return Err(format!("There was an error requesting the channel ID from the channel handle {}! Error: {error}", &channel_handle));
    }

    let list_result = result.unwrap().json::<ChannelListResponse>().await;
    if list_result.is_err() {
        let error = list_result.err().unwrap();
        return Err(format!("There was an error decoding the channel list response from the YouTube API! Error: {error}"));
    }

    let list_response = list_result.unwrap();
    assert!(list_response.items.len() > 0, "The specified channel handle has no associated channel!");
    
    let channel_id = list_response.items.get(0).unwrap().id.clone();
    request(format!("Requesting all videos from channel ID {}", &channel_id));
    let videos_request: Result<Vec<String>, String> = request_videos(VideosRequestParameters {
        channel_id: channel_id,
        api: api.clone(),
        next_page: None,
        previous_videos: None,
        include_streams_and_premieres: include_streams_and_premieres,
    }).await;

    if videos_request.is_err() {
        let error = videos_request.err().unwrap();
        return Err(format!("There was an error getting all videos on the specified channel! Error: {error}"));
    }

    Ok(videos_request.unwrap())
}

pub fn get_channel_handle_from_url(url: &String) -> String {
    assert!(url.find("@").is_some(), "Make sure to provide a valid YouTube Channel URL!");
    return String::from(url.split_at(url.find("@").unwrap() + 1).1);
}

#[allow(non_snake_case)] // needed for youtube api
#[derive(Deserialize)]
struct SearchListResponse {
    items: Vec<SearchResult>,
    nextPageToken: Option<String>,
}

#[allow(non_snake_case)] // needed for youtube api
#[derive(Deserialize)]
struct SearchResult {
    id: IDResponse,
}

#[allow(non_snake_case)] // needed for youtube api
#[derive(Deserialize)]
struct IDResponse {
    kind: String,
    videoId: Option<String>,
    // channelId: Option<String>,
}

struct VideosRequestParameters {
    channel_id: String,
    api: String,
    next_page: Option<String>,
    previous_videos: Option<Vec<String>>,
    include_streams_and_premieres: bool,
}

#[async_recursion]
async fn request_videos(params: VideosRequestParameters) -> Result<Vec<String>, String> {
    let mut videos: Vec<String>;
    if params.previous_videos.is_some() {
        videos = params.previous_videos.unwrap();
    } else {
        videos = Vec::new();
    }

    request(format!("Requesting an initial search for all videos from {}!", params.channel_id));
    let client = reqwest::Client::new();
    let mut initial_url = format!("{}/noKey/search?part=snippet,id&order=date&type=video&maxResults=50&channelId={}", params.api, params.channel_id);

    if params.next_page.is_some() {
        initial_url += format!("&pageToken={}", params.next_page.unwrap()).as_str();
    }

    let result = client.get(initial_url)
        .send().await;
    if result.is_err() {
        let error = result.err().unwrap();
        return Err(format!("There was an error requesting the channel videos from the channel id {}! Error: {error}", params.channel_id));
    }

    let search_parse_result = result.unwrap().json::<SearchListResponse>().await;
    if search_parse_result.is_err() {
        let error = search_parse_result.err().unwrap();
        return Err(format!("There was an error parsing the channel search results! Error: {error}"));
    }

    success(String::from("Got a search result from previous request! Parsing videos now."));
    let search_list = search_parse_result.unwrap();

    for search_result in search_list.items {
        if search_result.id.kind != "youtube#video" {
            // shouldn't be possible but just in case ig
            continue;
        }

        let id = &search_result.id.videoId.unwrap();
        if params.include_streams_and_premieres {
            videos.push(format!("https://youtu.be/{id}"));
            continue;
        }
        
        let is_stream_result = is_video_a_stream(id, &params.api).await;
        if is_stream_result.is_err() {
            let error = is_stream_result.err().unwrap();
            failure(format!("Failed to check if video was a livestream! Error: {error}"));
            continue;
        }

        if !is_stream_result.unwrap() {
            videos.push(format!("https://youtu.be/{id}"));
        }
    }

    if search_list.nextPageToken.is_some() {
        return request_videos(VideosRequestParameters {
            channel_id: params.channel_id,
            api: params.api,
            next_page: Some(search_list.nextPageToken.unwrap().clone()),
            previous_videos: Some(videos),
            include_streams_and_premieres: params.include_streams_and_premieres,
        }).await;
    }

    Ok(videos)
}

#[allow(non_snake_case)] // needed for youtube api
#[derive(Deserialize)]
struct VideoListResponse {
    items: Vec<VideoResult>,
}

#[allow(non_snake_case)] // needed for youtube api
#[derive(Deserialize)]
struct VideoResult {
    liveStreamingDetails: Option<LiveStreamingDetails>,
}

#[allow(non_snake_case)] // needed for youtube api
#[derive(Deserialize)]
struct LiveStreamingDetails {
    // actualStartTime: String,
    // actualEndTime: String,
}

async fn is_video_a_stream(id: &String, api: &String) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let result = client.get(format!("{api}/noKey/videos?part=liveStreamingDetails&id={id}"))
        .send().await;

    if result.is_err() {
        let error = result.err().unwrap();
        return Err(format!("There was an error requesting live stream details from video {}! Error: {error}", id));
    }

    let video_parse_result = result.unwrap().json::<VideoListResponse>().await;
    if video_parse_result.is_err() {
        let error = video_parse_result.err().unwrap();
        return Err(format!("There was an error parsing the video list! Error: {error}"));
    }

    let video_list = video_parse_result.unwrap();
    assert!(video_list.items.len() > 0, "Provide a valid YouTube video ID!");

    Ok(video_list.items[0].liveStreamingDetails.is_some())
}