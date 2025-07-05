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
mod files;

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

            if event.msg.content.ends_with("リセット") {
                instance.reset();
                say(&event.ctx, &event.msg, "履歴をリセットしたじゅう！").await;
            } else {
                Self::talk(&self.settings.api_key, instance, event).await?;
            }
        }
        Ok(())
    }

    async fn talk(api_key: &str, instance: &mut Chat, event: DiscordEvent) -> Result<(), Box<dyn Error>> {
        // println!("{:?}", &event.msg);
        let mut files = Vec::new();
        for attachment in &event.msg.attachments {
            // "text/plain; charset=utf-8"などのときに"text/plain"だけを抽出する
            let content_type = attachment.content_type.clone().unwrap();
            let content_type = content_type.split(';').next().unwrap().to_string();
            println!("{:?}", content_type);
            let buf = attachment.download().await?;
            let name = files::upload_bytes(api_key, &content_type, buf).await?;
            files.push(chat::FileData {
                mime_type: content_type,
                file_uri: name,
            })
        }
        let mut stream = instance.send_message(&event.msg.content, files).await?;
        let mut buf = String::new();
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
                say(&event.ctx, &event.msg, &buf).await;
                buf.clear();
            }
        }
        say(&event.ctx, &event.msg, &buf).await;
        buf.clear();
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
