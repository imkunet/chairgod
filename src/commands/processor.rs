use std::{mem, sync::Arc};

use anyhow::{bail, Context, Result};
use tracing::{info, warn};
use twilight_interactions::command::CreateCommand;
use twilight_model::{
    application::interaction::{application_command::CommandData, InteractionData},
    gateway::payload::incoming::InteractionCreate,
    oauth::Application,
};

use crate::{commands::admin::LFGDataCommand, models::ChairContext};

use super::admin::PingCommand;

pub async fn command_handle_interaction(
    mut interaction: Box<InteractionCreate>,
    context: Arc<ChairContext>,
) {
    let data = match mem::take(&mut interaction.data) {
        Some(InteractionData::ApplicationCommand(data)) => *data,
        _ => return,
    };

    if let Err(cause) = handle_command(*interaction, data, context).await {
        warn!(?cause, "failed to execute command");
    }
}

async fn handle_command(
    interaction: InteractionCreate,
    data: CommandData,
    context: Arc<ChairContext>,
) -> Result<()> {
    match data.name.as_str() {
        "ping" => PingCommand::handle(interaction, context).await?,
        "lfgdata" => LFGDataCommand::handle(interaction, data, context).await?,
        name => bail!("unknown command {name}"),
    }

    Ok(())
}

pub async fn register_commands(
    client: &twilight_http::Client,
    application: &Application,
) -> Result<()> {
    let commands = [
        PingCommand::create_command().into(),
        LFGDataCommand::create_command().into(),
    ];
    let interaction_client = client.interaction(application.id);

    let guilds = client
        .current_user_guilds()
        .await
        .context("fetching guilds")?
        .model()
        .await?;

    for guild in &guilds {
        interaction_client
            .set_guild_commands(guild.id, &commands)
            .await
            .with_context(|| format!("setting guild commands for {}", guild.id))?;
    }

    info!(
        "registered {} commands in {} guilds",
        commands.len(),
        guilds.len()
    );

    Ok(())
}
