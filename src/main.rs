mod lfg;
mod models;
mod util;

use anyhow::Result;
use tracing::info;
use tracing_panic::panic_hook;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    std::panic::set_hook(Box::new(panic_hook));
    info!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let db = sled::open("chairman.sled")?;
    let queues = db.open_tree("queues")?;

    queues.insert("sup", "folk")?;

    let a = queues.get("sup")?;
    match a {
        Some(v) => {
            let output = std::str::from_utf8(v.as_ref());
            info!("{:?}", output)
        }
        None => info!("not found :("),
    }

    Ok(())
}
