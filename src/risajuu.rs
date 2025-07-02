use std::collections::HashMap;
use std::error::Error;

use futures::{StreamExt, channel::mpsc::UnboundedReceiver};

use serenity::{
    client::Context,
    model::{
        channel::Message,
        id::{GuildId, UserId},
    },
};

use crate::discord::{DiscordEvent, PlaceIdentifier};

mod chat;
use chat::Chat;
pub use chat::ChatSettings;

type Receiver = UnboundedReceiver<DiscordEvent>;

#[derive(Hash, PartialEq, Eq, Clone)]
enum InstanceIdentifier {
    DM(UserId),
    Server(GuildId),
}

pub struct Risajuu {
    receiver: Receiver,
    chats: HashMap<InstanceIdentifier, Chat>,
    settings: ChatSettings,
}

impl Risajuu {
    pub fn new(receiver: Receiver, settings: ChatSettings) -> Risajuu {
        Risajuu {
            receiver,
            chats: HashMap::new(),
            settings,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut buf = String::new();
        while let Some(event) = self.receiver.next().await {
            let instance_key = match event.place {
                PlaceIdentifier::DM(_, user_id) => InstanceIdentifier::DM(user_id),
                PlaceIdentifier::Server(guild_id, _, _) => InstanceIdentifier::Server(guild_id),
            };
            let instance = match self.chats.get_mut(&instance_key) {
                Some(chat) => chat,
                None => {
                    self.chats.insert(instance_key.clone(), Chat::new(self.settings.clone()));
                    self.chats.get_mut(&instance_key).unwrap()
                }
            };

            let mut stream = instance.send_message(&event.msg.content).await?;
            while let Some(chunk) = stream.next().await {
                let reply = match chunk {
                    Ok(res) => res.candidates[0].content.parts[0].text.clone(),
                    Err(why) => Some(format!("\nGeminiのエラーじゅう。\n{}", why)),
                };
                //println!("{:?}", reply);
                if let Some(text) = reply {
                    buf.push_str(&text);
                    let v: Vec<&str> = buf.rsplitn(2, '\n').collect();
                    if let [s1, s2] = v[..] {
                        say(&event.ctx, &event.msg, s2).await;
                        buf = s1.to_string();
                    }
                } else {
                    say(&event.ctx, &event.msg, &mut buf).await;
                    buf.clear();
                }
            }
            say(&event.ctx, &event.msg, &mut buf).await;
            buf.clear();
        }
        Ok(())
    }
}

async fn say(ctx: &Context, msg: &Message, content: &str) {
    if !content.is_empty() {
        if let Err(why) = msg.channel_id.say(&ctx.http, content).await {
            eprintln!("Error (say): {:?}", why);
        }
    }
}
