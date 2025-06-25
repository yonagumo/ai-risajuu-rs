use std::error::Error;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use gemini_rs;

pub struct Discord {
    client: serenity::Client,
}

impl Discord {
    pub async fn new(token: &str, api_key: &str) -> Result<Discord, Box<dyn Error>> {
        let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
        let handler = Handler::new(api_key);
        let client = Client::builder(&token, intents).event_handler(handler).await?;
        Ok(Discord { client })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.client.start().await?;
        Ok(())
    }
}

struct Handler {
    gemini: gemini_rs::Client,
}

impl Handler {
    fn new(api_key: &str) -> Handler {
        let gemini = gemini_rs::Client::new(api_key);
        Handler { gemini }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if let Err(why) = msg.channel_id.broadcast_typing(&ctx.http).await {
            println!("Error (broadcast_typing): {:?}", why);
            return;
        }

        // let reply = "RustからHello, worldじゅう！";
        let mut route = self.gemini.generate_content("gemini-2.5-flash");
        route.message(&msg.content);
        let text = match route.await {
            Err(why) => Some(format!("Geminiのエラーじゅう。\n{}", why)),
            Ok(response) => response.candidates[0].content.parts[0].text.clone(),
        };

        if let Some(reply) = text {
            if let Err(why) = msg.channel_id.say(&ctx.http, reply).await {
                println!("Error (say): {:?}", why);
            }
        }
    }
}
