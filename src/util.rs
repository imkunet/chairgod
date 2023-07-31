use anyhow::{Context, Result};
use tracing::warn;
use twilight_model::channel::message::Embed;
use twilight_util::builder::embed::EmbedBuilder;

pub fn simple_embed(color: u32, title: &str, desc: &str) -> Result<Embed> {
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

// really REALLY dirty method
pub fn coerce_into_u64(slice: &[u8]) -> u64 {
    if slice.len() < 8 {
        warn!("incomplete &[u8] trying to be coerced into u64 {:?}", slice);
        return 0;
    }

    let mut buf = [0u8; 8];
    buf[..8].copy_from_slice(&slice[..8]);
    u64::from_be_bytes(buf)
}
