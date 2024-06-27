/*
    File used internally to abstract HTTP requests just that bit more, and to reduce redundancy.
*/
use reqwest::{Client, Error, Response};
use serde::Serialize;

pub(crate) struct GetRequest {
    pub url: String,
    pub accept: Option<String>,
}

pub(crate) async fn get_request(request: GetRequest, client: &Client) -> Result<Response, Error> {
    let mut get_builder = client.get(request.url);
    if request.accept.is_some() {
        get_builder = get_builder.header("Accept", request.accept.unwrap());
    }

    return get_builder.send().await;
}

pub(crate) struct PostJSONRequest<T> {
    pub url: String,
    pub accept: Option<String>,
    pub json: T,
}

pub(crate) async fn post_json_request<T: Serialize>(
    request: PostJSONRequest<T>,
    client: &Client,
) -> Result<Response, Error> {
    let mut post_builder = client
        .post(request.url)
        .header("Content-Type", "application/json")
        .json(&request.json);
    if request.accept.is_some() {
        post_builder = post_builder.header("Accept", request.accept.unwrap());
    }

    return post_builder.send().await;
}
