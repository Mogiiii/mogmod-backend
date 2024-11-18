use axum::{extract::State, http::StatusCode, Json};
use log::{debug, error, warn};
use serde::Deserialize;
use std::{sync::Arc, time::SystemTime};

use crate::persistence;
use crate::transformer;

#[derive(Clone)]
pub(crate) struct Context {
    pub(crate) db_client: Arc<tokio_postgres::Client>,
}

#[derive(Deserialize)]
pub(crate) struct IncomingMessage {
    id: i64,
    content: String,
    timestamp: SystemTime,
    user_id: i64,
    user_name: String,
    guild_id: i64,
    guild_name: String,
    channel_id: i64,
    channel_name: String,
    edited_timestamp: Option<SystemTime>,
}

pub(crate) async fn get_guilds(State(context): State<Context>) -> (StatusCode, Json<Vec<String>>) {
    debug!("getting guilds");
    let r = persistence::get_guilds(&context.db_client).await;
    match r {
        Ok(v) => (StatusCode::OK, Json(v)),
        Err(e) => {
            error!("error getting guilds: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
        }
    }
}

pub(crate) async fn get_users(State(context): State<Context>) -> (StatusCode, Json<Vec<String>>) {
    debug!("getting users");
    let r = persistence::get_users(&context.db_client).await;
    match r {
        Ok(v) => (StatusCode::OK, Json(v)),
        Err(e) => {
            error!("error getting users: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
        }
    }
}

pub(crate) async fn get_messages(
    State(context): State<Context>,
    user_name: String,
) -> (StatusCode, Json<Vec<persistence::Message>>) {
    debug!("getting messages from {user_name}");
    let r = persistence::get_messages(&context.db_client, &user_name).await;
    match r {
        Ok(v) => (StatusCode::OK, Json(v)),
        Err(e) => {
            error!("error getting messages: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
        }
    }
}

pub(crate) async fn post_message(
    State(context): State<Context>,
    message: Json<IncomingMessage>,
) -> (StatusCode, String) {
    debug!("updating message: {}", message.id);
    let usr = persistence::User {
        id: message.user_id,
        name: message.user_name.clone(),
    };

    let guild = persistence::Guild {
        id: message.guild_id,
        name: message.guild_name.clone(),
        deleted: false,
    };

    let channel = persistence::Channel {
        id: message.channel_id,
        name: message.channel_name.clone(),
        guild_id: message.guild_id,
        deleted: false,
    };

    let mut msg = persistence::Message {
        id: message.id,
        content: message.content.clone(),
        timestamp: message.timestamp,
        user_id: message.user_id,
        guild_id: message.guild_id,
        channel_id: message.channel_id,
        sentiment: None,
        sentiment_confidence: None,
        edited_timestamp: message.edited_timestamp,
        deleted: false,
    };

    match transformer::get_sentiment(&msg.content).await {
        Ok(s) => {
            msg.sentiment = Some(s.label);
            msg.sentiment_confidence = Some(s.score);
        }
        Err(e) => warn!("error while getting sentiment: {e}"),
    }

    if let Err(e) = persistence::update_user(&context.db_client, &usr).await {
        error!("error updating user: {e}\n{usr:?}");
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"));
    }

    if let Err(e) = persistence::update_guild(&context.db_client, &guild).await {
        error!("error updating guild: {e}\n{guild:?}");
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"));
    }

    if let Err(e) = persistence::update_channel(&context.db_client, &channel).await {
        error!("error updating channel: {e}\n{channel:?}");
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"));
    }

    match persistence::update_message(&context.db_client, &msg).await {
        Ok(_) => (StatusCode::OK, "OK".to_string()),
        Err(e) => {
            error!("error updating message: {e}\n{msg:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))
        }
    }
}
