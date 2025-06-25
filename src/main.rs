use dotenv::dotenv;
use std::env;
use std::error::Error;

mod discord;

use discord::Discord;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv()?;
    let token = env::var("DISCORD_TOKEN")?;
    let api_key = env::var("GOOGLE_API_KEY")?;
    let mut discord = Discord::new(&token, &api_key).await?;
    discord.run().await?;
    Ok(())
}
