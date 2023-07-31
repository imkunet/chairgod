use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::{TimeZone, Utc};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::{
    builder::{embed::EmbedBuilder, InteractionResponseDataBuilder},
    snowflake::Snowflake,
};

use crate::models::ChairContext;

#[derive(CommandModel, CreateCommand)]
#[command(name = "ping", desc = "Check the latency of the bot")]
pub struct PingCommand;

impl PingCommand {
    pub async fn handle(interaction: InteractionCreate, context: Arc<ChairContext>) -> Result<()> {
        let display_latency = match context.latency.average() {
            Some(v) => format!("{:#?}", v),
            None => "Unknown".to_string(),
        };

        let sent_time = Utc
            .timestamp_millis_opt(interaction.id.timestamp())
            .earliest();

        let display_roundtrip = match sent_time {
            Some(v) => {
                let duration = Utc::now() - v;
                format!(
                    "{:#?}",
                    duration
                        .to_std()
                        .context("failure in timestamp conversion")?
                )
            }
            None => "Error in time conversion".to_string(),
        };

        let embed = EmbedBuilder::new()
            .color(0x85db5e)
            .title("Ping")
            .description(format!("`•` The time it takes for the bot to talk to Discord is `{}` (exact)
                            `•` The time it took for this command to execute roundtrip is `{}` (approximation)", display_latency, display_roundtrip))
                            .build();

        let client = context.http.interaction(interaction.application_id);
        let data = InteractionResponseDataBuilder::new()
            .embeds([embed])
            .build();

        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        client
            .create_response(interaction.id, &interaction.token, &response)
            .await?;

        Ok(())
    }
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "lfgdata", desc = "Manage LFG types [KuNet only]")]
pub enum LFGDataCommand {
    #[command(name = "list")]
    List(LFGList),
    #[command(name = "add")]
    Add(LFGAdd),
    #[command(name = "remove")]
    Remove(LFGRemove),
}

impl LFGDataCommand {
    pub async fn handle(
        interaction: InteractionCreate,
        data: CommandData,
        context: Arc<ChairContext>,
    ) -> Result<()> {
        let command =
            LFGDataCommand::from_interaction(data.into()).context("parsing command data")?;

        match command {
            LFGDataCommand::List(command) => command.run(interaction, context).await,
            LFGDataCommand::Add(command) => command.run(interaction, context).await,
            LFGDataCommand::Remove(command) => command.run(interaction, context).await,
        }
    }
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "list", desc = "List the LFG types")]
pub struct LFGList;

impl LFGList {
    pub async fn run(
        &self,
        interaction: InteractionCreate,
        context: Arc<ChairContext>,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "add", desc = "Add to the LFG types")]
pub struct LFGAdd {
    /// The facade role id
    pub facade: String,
    /// The actual role id
    pub actual: String,
}

impl LFGAdd {
    pub async fn run(
        &self,
        interaction: InteractionCreate,
        context: Arc<ChairContext>,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "remove", desc = "Remove from the LFG types")]
pub struct LFGRemove {
    /// The facade role id
    pub facade: String,
}

impl LFGRemove {
    pub async fn run(
        &self,
        interaction: InteractionCreate,
        context: Arc<ChairContext>,
    ) -> Result<()> {
        Ok(())
    }
}
