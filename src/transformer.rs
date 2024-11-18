use std::env;

use log::debug;
use reqwest::Error;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub(crate) struct Sentiment {
    pub(crate) label: String,
    pub(crate) score: f32,
}

#[derive(Serialize)]
struct TransformerRequest {
    text: String,
}

pub(crate) async fn get_sentiment(content: &str) -> Result<Sentiment, Error> {
    let base_url = env::var("TRANSFORMER_API").expect("Missing Env var: TRANSFORMER_API");
    let client = reqwest::Client::new();

    debug!("GET {base_url}/sentiment | {content}");
    let tr = TransformerRequest {
        text: content.to_string(),
    };
    let r = client
        .get(format!("{base_url}/sentiment"))
        .json(&tr)
        .send()
        .await?;
    let s: Sentiment = r.json().await?;
    debug!("got sentiment for \"{content}\": {s:?}");
    Ok(s)
}
