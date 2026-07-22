mod config;
mod embed_color;
mod mention;
mod send;

pub use config::DiscordConfig;
pub use embed_color::EmbedColor;
pub use send::{DiscordMessageInput, send_discord_message};
