mod archive;
mod restore;

use archive::archive;
pub use archive::archive_soundboard;
pub use restore::{detect_audio_mime, perform_restore};

#[poise::command(slash_command, subcommands("archive"))]
pub async fn soundboard(
    _: crate::Context<'_>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Ok(())
}
