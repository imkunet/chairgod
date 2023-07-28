mod config;
mod lfg;
mod models;
mod util;

use std::sync::Arc;

use anyhow::Result;
use tracing::{error, info, warn};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Event, Intents, Latency, Shard, ShardId};

use crate::{config::ChairConfig, lfg::LFGManager, models::ChairContext};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install().expect("unable to setup panic logging");
    info!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(_) => info!("could not load .env, skipping...."),
    };

    let config = match ChairConfig::try_load_from_env() {
        Ok(v) => v,
        Err(_) => {
            error!("environment variables not found!");
            return Ok(());
        }
    };

    let token = config.bot_token;
    let intents = Intents::GUILDS | Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;

    let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);
    let http = Arc::new(twilight_http::Client::new(token));

    let cache = Arc::new(
        InMemoryCache::builder()
            .resource_types(ResourceType::MESSAGE)
            .build(),
    );

    let lfg_manager = Arc::new(LFGManager::new());

    loop {
        let event = match shard.next_event().await {
            Ok(v) => v,
            Err(cause) => {
                warn!(?cause, "error receiving event");

                if cause.is_fatal() {
                    error!("fatal error!");
                    break;
                }

                continue;
            }
        };

        cache.update(&event);

        let context = ChairContext {
            http: http.clone(),
            cache: cache.clone(),
            latency: shard.latency().clone(),
            lfg: lfg_manager.clone(),
        };
        tokio::spawn(handle_event(event, context));
    }

    Ok(())
}

async fn handle_event(event: Event, context: ChairContext) -> Result<()> {
    Ok(())
}
