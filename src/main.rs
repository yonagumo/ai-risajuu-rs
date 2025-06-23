use dotenv::dotenv;
use std::collections::HashMap;
use std::env;
use std::error::Error;

use pyo3::{prelude::*, types::IntoPyDict};
use serenity::{async_trait, model::channel::Message, model::gateway::Ready, prelude::*};

struct Handler {
    google_api_key: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        // let reply = "RustからHello, worldじゅう！";
        let reply: String = Python::with_gil(|py| {
            let genai = py.import("google.genai")?;
            let args = [("api_key", self.google_api_key.clone())].into_py_dict(py)?;
            let client = genai.call_method("Client", (), Some(&args))?;
            let mut args = HashMap::<&str, &str>::new();
            args.insert("model", "gemini-2.5-flash");
            args.insert("contents", &msg.content);
            let response =
                client
                    .getattr("models")?
                    .call_method("generate_content", (), Some(&args.into_py_dict(py)?))?;
            Ok::<String, Box<dyn Error>>(response.getattr("text")?.to_string())
        })
        .unwrap();

        if let Err(why) = msg.channel_id.say(&ctx.http, reply).await {
            println!("Error sending message: {:?}", why);
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv()?;
    let token = env::var("DISCORD_TOKEN")?;
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let handler = Handler {
        google_api_key: env::var("GOOGLE_API_KEY")?,
    };
    let mut client = Client::builder(&token, intents).event_handler(handler).await?;
    client.start().await?;
    Ok(())
}
