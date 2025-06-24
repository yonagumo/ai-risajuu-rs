use dotenv::dotenv;
use std::env;
use std::error::Error;

use gemini_rs::{self as gemini};
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

struct Handler {
    client: gemini_rs::Client,
}

impl Handler {
    fn new(api_key: &str) -> Handler {
        let client = gemini::Client::new(api_key);
        Handler { client }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if let Err(why) = msg.channel_id.broadcast_typing(&ctx.http).await {
            println!("Error in broadcast_typing: {:?}", why);
            return;
        }

        // let reply = "RustからHello, worldじゅう！";
        let mut route = self.client.generate_content("gemini-2.5-flash");
        route.message(&msg.content);
        let text = match route.await {
            Err(why) => Some(format!("エラーじゅう：{}", why)),
            Ok(response) => response.candidates[0].content.parts[0].text.clone(),
        };

        if let Some(reply) = text {
            if let Err(why) = msg.channel_id.say(&ctx.http, reply).await {
                println!("Error sending message: {:?}", why);
            }
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
    let handler = Handler::new(&env::var("GOOGLE_API_KEY")?);
    let mut client = serenity::Client::builder(&token, intents).event_handler(handler).await?;
    client.start().await?;
    Ok(())
}
