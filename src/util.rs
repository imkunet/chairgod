use anyhow::{Context, Result};
use twilight_model::channel::message::Embed;
use twilight_util::builder::embed::EmbedBuilder;

pub(crate) fn simple_embed(color: u32, title: &str, desc: &str) -> Result<Embed> {
    Ok(EmbedBuilder::new()
        .color(color)
        .title(title)
        .description(desc)
        .validate()
        .with_context(|| {
            format!(
                "failure to validate simple embed {}/{}/{}",
                color, title, desc
            )
        })?
        .build())
}
