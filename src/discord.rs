use futures::channel::mpsc::UnboundedSender;
use std::error::Error;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

type Sender = UnboundedSender<(Context, Message)>;

pub struct Discord {
    client: Client,
}

impl Discord {
    pub async fn new(token: &str, sender: Sender) -> Result<Discord, Box<dyn Error>> {
        let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
        let handler = Handler::new(sender);
        let client = Client::builder(&token, intents).event_handler(handler).await?;
        Ok(Discord { client })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.client.start().await?;
        Ok(())
    }
}

struct Handler {
    sender: Sender,
}

impl Handler {
    fn new(sender: Sender) -> Handler {
        Handler { sender }
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

        self.sender.unbounded_send((ctx, msg)).unwrap();
    }
}
