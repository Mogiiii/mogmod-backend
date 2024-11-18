use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::{env, time::SystemTime};
use tokio_postgres::{types::ToSql, Client, Error, NoTls};

#[derive(Serialize, Debug)]
pub(crate) struct Message {
    pub(crate) id: i64,
    pub(crate) content: String,
    pub(crate) timestamp: SystemTime,
    pub(crate) user_id: i64,
    pub(crate) guild_id: i64,
    pub(crate) channel_id: i64,
    pub(crate) sentiment: Option<String>,
    pub(crate) sentiment_confidence: Option<f32>,
    pub(crate) edited_timestamp: Option<SystemTime>,
    pub(crate) deleted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct User {
    pub(crate) id: i64,
    pub(crate) name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Guild {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) deleted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Channel {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) guild_id: i64,
    pub(crate) deleted: bool,
}

//TODO: reconnect if it disconnects
pub(crate) async fn setup() -> Result<Client, Error> {
    let hostname = env::var("PG_HOSTNAME").expect("Missing Env var: PG_HOSTNAME");
    let user = env::var("PG_USER").expect("Missing Env var: PG_USER");
    let dbname = env::var("PG_DBNAME").expect("Missing Env var: PG_DBNAME");

    info!("Connecting to postgres: host={hostname} user={user} dbname={dbname}");
    let (client, connection) = tokio_postgres::connect(
        &format!("host={hostname} user={user} dbname={dbname}"),
        NoTls,
    )
    .await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    info!("Successfully connected to postgres");
    Ok(client)
}

pub(crate) async fn get_guilds(client: &Client) -> Result<Vec<String>, Error> {
    let query = "SELECT name FROM guilds";
    let rows = client.query(query, &[]).await?;
    let names = rows
        .iter()
        .map(|r| r.get::<usize, &str>(0).to_string())
        .collect();
    Ok(names)
}

pub(crate) async fn get_users(client: &Client) -> Result<Vec<String>, Error> {
    let query = "SELECT name FROM users";
    let rows = client.query(query, &[]).await?;
    let names = rows
        .iter()
        .map(|r| r.get::<usize, &str>(0).to_string())
        .collect();
    Ok(names)
}

pub(crate) async fn get_messages(client: &Client, user_name: &str) -> Result<Vec<Message>, Error> {
    let query = "SELECT
                        messages.id,
                        messages.content,
                        messages.timestamp,
                        messages.channel_id,
                        messages.guild_id,
                        messages.user_id,
                        messages.sentiment,
                        messages.sentiment_confidence,
                        messages.edited_timestamp,
                        messages.deleted,
                        users.name 
                    FROM messages 
                    join users 
                        on messages.user_id = users.id 
                    WHERE users.name = $1";
    let rows = client.query(query, &[&user_name]).await?;
    let msgs = rows
        .iter()
        .map(|r| Message {
            id: r.get(0),
            content: r.get(1),
            timestamp: r.get(2),
            channel_id: r.get(3),
            guild_id: r.get(4),
            user_id: r.get(5),
            sentiment: r.get(6),
            sentiment_confidence: r.get(7),
            edited_timestamp: r.get(8),
            deleted: r.get(9),
        })
        .collect();
    Ok(msgs)
}

//update user
async fn create_user(client: &Client, user: &User) -> Result<bool, Error> {
    let statement = "INSERT INTO users (id, name) VALUES ($1, $2)";
    client.execute(statement, &[&user.id, &user.name]).await?;
    Ok(true)
}

pub(crate) async fn update_user(client: &Client, user: &User) -> Result<bool, Error> {
    let statement = "UPDATE users SET name = $2 WHERE id = $1";
    let rows_modified = client.execute(statement, &[&user.id, &user.name]).await?;
    match rows_modified {
        0 => create_user(&client, &user).await,
        1 => Ok(true),
        x => {
            warn!("Modified too many rows updating users: {x} rows | {user:?}");
            Ok(false)
        }
    }
}

async fn create_guild(client: &Client, guild: &Guild) -> Result<bool, Error> {
    let statement = "INSERT INTO guilds (id, name) VALUES ($1, $2)";
    client.execute(statement, &[&guild.id, &guild.name]).await?;
    Ok(true)
}

pub(crate) async fn update_guild(client: &Client, guild: &Guild) -> Result<bool, Error> {
    let statement = "UPDATE guilds SET name = $1, deleted = $2 WHERE id = $3";
    let rows_modified = client.execute(statement, &[&guild.id, &guild.deleted, &guild.name]).await?;
    match rows_modified {
        0 => create_guild(&client, &guild).await,
        1 => Ok(true),
        x => {
            warn!("Modified too many rows updating guilds: {x} rows | {guild:?}");
            Ok(false)
        }
    }
}

async fn create_channel(client: &Client, channel: &Channel) -> Result<bool, Error> {
    let statement = "INSERT INTO channels (id, name, guild_id) VALUES ($1, $2, $3)";
    client
        .execute(statement, &[&channel.id, &channel.name, &channel.guild_id])
        .await?;
    Ok(true)
}

pub(crate) async fn update_channel(client: &Client, channel: &Channel) -> Result<bool, Error> {
    let statement = "UPDATE channels SET name = $1, deleted = $2 WHERE id = $3";
    let rows_modified = client
        .execute(statement, &[&channel.name, &channel.deleted, &channel.id])
        .await?;
    match rows_modified {
        0 => create_channel(&client, &channel).await,
        1 => Ok(true),
        x => {
            warn!("Modified too many rows updating channels: {x} rows | {channel:?}");
            Ok(false)
        }
    }
}

pub(crate) async fn create_message(client: &Client, message: &Message) -> Result<bool, Error> {
    let statement = "INSERT INTO messages (id, user_id, guild_id, channel_id, content, timestamp, sentiment, sentiment_confidence)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";
    client
        .execute(
            statement,
            &[
                &message.id,
                &message.user_id,
                &message.guild_id,
                &message.channel_id,
                &message.content,
                &message.timestamp,
                &message.sentiment,
                &message.sentiment_confidence,
            ],
        )
        .await?;
    Ok(true)
}

pub(crate) async fn update_message(client: &Client, message: &Message) -> Result<bool, Error> {
    let mut index = 1;
    let mut statement = format!("UPDATE messages SET content = ${index}");
    let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();
    params.push(&message.content);
    index += 1;

    if let Some(s) = &message.sentiment {
        statement = format!("{statement}, sentiment = ${index}");
        index += 1;
        params.push(s);
    }

    if let Some(c) = &message.sentiment_confidence {
        statement = format!("{statement}, sentiment_confidence = ${index}");
        index += 1;
        params.push(c);
    }

    if let Some(t) = &message.edited_timestamp {
        statement = format!("{statement}, edited_timestamp = ${index}");
        index += 1;
        params.push(t);
    }

    if message.deleted {
        statement = format!("{statement}, deleted = ${index}");
        index += 1;
        params.push(&message.deleted);
    }

    statement = format!("{statement} WHERE id = ${index}");
    params.push(&message.id);
    let rows_modified = client.execute(&statement, &params).await?;
    match rows_modified {
        0 => create_message(&client, &message).await,
        1 => Ok(true),
        x => {
            warn!("Modified too many rows updating messages: {x} rows | {message:?}");
            Ok(false)
        }
    }
}
