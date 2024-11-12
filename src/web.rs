use axum::{extract::State, http::StatusCode, Json};
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
}

pub(crate) async fn get_guilds(State(context): State<Context>) -> (StatusCode, Json<Vec<String>>) {
    let r = persistence::get_guilds(&context.db_client).await;
    match r {
        Ok(v) => (StatusCode::OK, Json(v)),
        Err(e) => {
            println!("{e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
        }
    }
}

pub(crate) async fn get_users(State(context): State<Context>) -> (StatusCode, Json<Vec<String>>) {
    let r = persistence::get_users(&context.db_client).await;
    match r {
        Ok(v) => (StatusCode::OK, Json(v)),
        Err(e) => {
            println!("{e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
        }
    }
}

pub(crate) async fn get_messages(
    State(context): State<Context>,
    user_name: String
) -> (StatusCode, Json<Vec<persistence::Message>>) {
    let r = persistence::get_messages(&context.db_client, &user_name).await;
    match r {
        Ok(v) => (StatusCode::OK, Json(v)),
        Err(e) => {
            println!("{e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
        }
    }
}

pub(crate) async fn post_message(
    State(context): State<Context>,
    message: Json<IncomingMessage>,
) -> (StatusCode, String) {
    let usr = persistence::User {
        id: message.user_id,
        name: message.user_name.clone(),
    };

    let guild = persistence::Guild {
        id: message.guild_id,
        name: message.guild_name.clone(),
    };

    let channel = persistence::Channel {
        id: message.channel_id,
        name: message.channel_name.clone(),
        guild_id: message.guild_id,
    };

    let mut msg = persistence::Message {
        id: message.id,
        content: message.content.clone(),
        timestamp: message.timestamp,
        user_id: message.user_id,
        guild_id: message.guild_id,
        channel_id: message.channel_id,
        sentiment: "unknown".to_string(), //default value in case transformer api is down
        sentiment_confidence: 0.0,
    };

    match transformer::get_sentiment(&msg.content).await {
        Ok(s) => {
            //println!("successfully got sentiment: {:?}", s);
            msg.sentiment = s.label;
            msg.sentiment_confidence = s.score;
        }
        Err(e) => println!("{e}")
    }
    //println!("after getting sentiment: {:?}", msg);

    if let Err(e) = persistence::update_user(&context.db_client, &usr).await {
        println!("{e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"));
    }

    if let Err(e) = persistence::update_guild(&context.db_client, &guild).await {
        println!("{e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"));
    }

    if let Err(e) = persistence::update_channel(&context.db_client, &channel).await {
        println!("{e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"));
    }

    match persistence::update_message(&context.db_client, &msg).await {
        Ok(_) => (StatusCode::OK, "OK".to_string()),
        Err(e) => {
            println!("{e}");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))
        }
    }
}