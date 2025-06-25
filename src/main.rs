use dotenv::dotenv;
use futures::{channel::mpsc, try_join};
use std::env;
use std::error::Error;

mod discord;
mod risajuu;

use discord::Discord;
use risajuu::Risajuu;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv()?;

    let (sender, receiver) = mpsc::unbounded();

    let token = env::var("DISCORD_TOKEN")?;
    let mut discord = Discord::new(&token, sender).await?;

    let api_key = env::var("GOOGLE_API_KEY")?;
    let model_name = env::var("AI_MODEL")?;
    let mut risajuu = Risajuu::new(&api_key, receiver, model_name);

    try_join!(discord.run(), risajuu.run())?;
    Ok(())
}
