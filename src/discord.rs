use futures::channel::mpsc::UnboundedSender;
use std::error::Error;

use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::Ready,
        id::{ChannelId, GuildId, UserId},
    },
    prelude::*,
};

type Sender = UnboundedSender<(Context, Message)>;

pub struct Discord {
    client: Client,
}

impl Discord {
    pub async fn new(token: &str, sender: Sender, targets_str: &str) -> Result<Discord, Box<dyn Error>> {
        //let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
        //let intents = GatewayIntents::default();
        let intents = GatewayIntents::all();
        let targets = targets_str
            .split(',')
            .map(|sc| {
                let sc: Vec<&str> = sc.split('/').collect();
                (sc[0].to_string(), sc[1].to_string())
            })
            .collect();
        let handler = Handler::new(sender, targets);
        let client = Client::builder(&token, intents).event_handler(handler).await?;
        Ok(Discord { client })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.client.start().await?;
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum PlaceIdentifier {
    DM(ChannelId, UserId),
    Server(GuildId, Option<ChannelId>, ChannelId),
}

#[allow(dead_code)]
impl PlaceIdentifier {
    async fn new(ctx: &Context, msg: &Message) -> PlaceIdentifier {
        if let Some(guild_id) = msg.guild_id {
            let category = msg.category_id(&ctx.http).await;
            Self::Server(guild_id, category, msg.channel_id)
        } else {
            PlaceIdentifier::DM(msg.channel_id, msg.author.id)
        }
    }

    fn is_dm(&self) -> bool {
        match self {
            Self::DM(_, _) => true,
            _ => false,
        }
    }

    fn is_server(&self) -> bool {
        match self {
            Self::Server(_, _, _) => true,
            _ => false,
        }
    }
}

struct Handler {
    sender: Sender,
    targets: Vec<(String, String)>,
}

impl Handler {
    fn new(sender: Sender, targets: Vec<(String, String)>) -> Handler {
        Handler { sender, targets }
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

        let place = PlaceIdentifier::new(&ctx, &msg).await;

        let is_target = match place {
            PlaceIdentifier::DM(_, _) => true,
            PlaceIdentifier::Server(guild, _, channel) => {
                let guild_name = guild.name(&ctx.cache).unwrap();
                let channel_name = channel.name(&ctx.http).await.unwrap();
                self.targets.contains(&(guild_name, channel_name))
            }
        };

        if is_target {
            if let Err(why) = msg.channel_id.broadcast_typing(&ctx.http).await {
                eprintln!("Error (broadcast_typing): {:?}", why);
                return;
            }

            self.sender.unbounded_send((ctx, msg)).unwrap();
        }
    }
}
