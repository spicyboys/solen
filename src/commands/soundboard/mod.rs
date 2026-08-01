mod archive;
mod list;
mod restore;

use archive::archive;
pub use archive::archive_soundboard;
pub use list::build_list_components;
use list::list;
pub use restore::perform_restore;
use restore::restore;

#[poise::command(slash_command, subcommands("archive", "list", "restore"))]
pub async fn soundboard(
    _: crate::Context<'_>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Ok(())
}
