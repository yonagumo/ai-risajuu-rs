use dotenv::dotenv;
use futures::{channel::mpsc, try_join};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Read;

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
    let system_instruction = load_text("src/system_instruction.md")?;
    let mut risajuu = Risajuu::new(&api_key, receiver, model_name, system_instruction);

    try_join!(discord.run(), risajuu.run())?;
    Ok(())
}

fn load_text(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(text)
}
