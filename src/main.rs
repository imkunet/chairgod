mod commands;
mod config;
mod lfg;
mod models;
mod util;

use std::sync::Arc;

use anyhow::{Context, Result};
use commands::processor::command_handle_interaction;
use tracing::{error, info, warn};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Event, Intents, Shard, ShardId};

use crate::{
    commands::processor::register_commands, config::ChairConfig, lfg::LFGManager,
    models::ChairContext,
};

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

    let db = sled::open("chair.sled")?;
    /*let test = db.open_tree("mention_types")?;
    test.insert(
        (1069644995780423731_u64).to_be_bytes(),
        &(1069645017414631474_u64).to_be_bytes(),
    )?;*/

    let token = config.bot_token;
    let intents = Intents::GUILDS | Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;

    let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);
    let http = Arc::new(twilight_http::Client::new(token));

    let application = http.current_user_application().await?.model().await?;
    let application_id = application.id;

    info!("logged in as {} ({})", application.name, application_id);

    register_commands(&http, &application)
        .await
        .context("registering commands")?;

    let cache = Arc::new(
        InMemoryCache::builder()
            .resource_types(ResourceType::MESSAGE)
            .build(),
    );

    let lfg_manager = Arc::new(LFGManager::new(&db).context("creating lfg")?);

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
            application_id,
            cache: cache.clone(),
            latency: shard.latency().clone(),
            lfg: lfg_manager.clone(),
        };

        tokio::spawn(async move {
            if let Err(cause) = handle_event(event, context).await {
                warn!(?cause, "error in handling event")
            }
        });
    }

    Ok(())
}

async fn handle_event(event: Event, context: ChairContext) -> Result<()> {
    let context = Arc::new(context);
    match event {
        Event::MessageCreate(msg) => {
            context.lfg.on_message(context.clone(), msg).await?;
        }
        Event::MessageUpdate(msg) => {
            info!("sup");
        }
        Event::InteractionCreate(interaction) => {
            command_handle_interaction(interaction.clone(), context.clone()).await;
        }
        _ => {}
    }

    Ok(())
}
