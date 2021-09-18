//! Replies with "hi" to "hello", without any
//! handlers and context passed around.
//!
//! Base: example-self-bot
//! Book chapter: None

use std::sync::Arc;

use robespierre_cache::{Cache, CacheConfig, CommitToCache};
use robespierre_events::{Authentication, Connection};
use robespierre_http::{Http, HttpAuthentication};
use robespierre_models::{
    channels::{Message, MessageContent},
    events::ServerToClientEvent,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");

    let http = Http::new(HttpAuthentication::UserSession {
        session_token: &token,
    })
    .await?;

    let http = Arc::new(http);

    let mut connection = Connection::connect(Authentication::User {
        session_token: &token,
    })
    .await?;

    let cache = Cache::new(CacheConfig::default());

    let acc = http.fetch_account().await?;

    loop {
        let event = connection.next().await?;

        event.commit_to_cache_ref(&cache).await;

        // maintain the server list
        match &event {
            ServerToClientEvent::ServerMemberJoin { id, user } => {
                if *user == acc.id {
                    http.fetch_server(*id).await?.commit_to_cache(&cache).await;
                }
            }
            ServerToClientEvent::ServerMemberLeave { id, user } => {
                if *user == acc.id {
                    cache.delete_server(*id).await;
                }
            }
            _ => {}
        }

        tokio::spawn(handle(event, Arc::clone(&http)));
    }
}

async fn handle(event: ServerToClientEvent, http: Arc<Http>) {
    if let ServerToClientEvent::Message {
        message:
            message
            @
            Message {
                content: MessageContent::Content(s),
                ..
            },
    } = &event
    {
        if s == "hello" {
            let _ = http
                .send_message(
                    message.channel,
                    "hi",
                    rusty_ulid::generate_ulid_string(),
                    vec![],
                    vec![],
                )
                .await;
        }
    }
}
