use std::error::Error;

use futures::{StreamExt, channel::mpsc::UnboundedReceiver};

use serenity::{client::Context, model::channel::Message};

mod chat;
use chat::Chat;
pub use chat::ChatSettings;

type Receiver = UnboundedReceiver<(Context, Message)>;

pub struct Risajuu {
    receiver: Receiver,
    chat: Chat,
}

impl Risajuu {
    pub fn new(receiver: Receiver, settings: ChatSettings) -> Risajuu {
        Risajuu {
            receiver,
            chat: Chat::new(settings),
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut buf = String::new();
        while let Some((ctx, msg)) = self.receiver.next().await {
            let mut stream = self.chat.send_message(&msg.content).await?;
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
                        say(&ctx, &msg, s2).await;
                        buf = s1.to_string();
                    }
                } else {
                    say(&ctx, &msg, &mut buf).await;
                    buf.clear();
                }
            }
            say(&ctx, &msg, &mut buf).await;
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
