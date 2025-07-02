use dotenv::dotenv;
use futures::{channel::mpsc, try_join};
use std::{env, error::Error, fs::File, io, io::Read};

mod discord;
mod risajuu;

use discord::Discord;
use risajuu::{ChatSettings, Risajuu};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv()?;

    let (sender, receiver) = mpsc::unbounded();

    let token = env::var("DISCORD_TOKEN")?;
    let targets_str = env::var("TARGET_CHANNEL")?;
    let mut discord = Discord::new(&token, sender, &targets_str).await?;

    let settings = ChatSettings {
        api_key: env::var("GOOGLE_API_KEY")?,
        model_name: env::var("AI_MODEL")?,
        system_instruction: load_text("src/system_instruction.md")?,
    };
    let mut risajuu = Risajuu::new(receiver, settings);

    try_join!(discord.run(), risajuu.run())?;
    Ok(())
}

fn load_text(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(text)
}
