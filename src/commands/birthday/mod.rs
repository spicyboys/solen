mod register;
mod remove;

use register::register;
use remove::remove;

#[poise::command(slash_command, subcommands("register", "remove"))]
pub async fn birthday(_: crate::Context<'_>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Ok(())
}
