use std::error::Error;

use futures::channel::mpsc::UnboundedReceiver;

use futures::StreamExt;
use gemini_rs::Client;
use gemini_rs::Error as GeminiError;
use serenity::{client::Context, model::channel::Message};

type Receiver = UnboundedReceiver<(Context, Message)>;

pub struct Risajuu {
    receiver: Receiver,
    gemini: Client,
    model_name: String,
}

impl Risajuu {
    pub fn new(api_key: &str, receiver: Receiver, model_name: String) -> Risajuu {
        let gemini = gemini_rs::Client::new(api_key);
        Risajuu {
            receiver,
            gemini,
            model_name,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        while let Some((ctx, msg)) = self.receiver.next().await {
            let response = self.chat(&msg.content).await;
            let reply = match response {
                Err(why) => Some(format!("Geminiのエラーじゅう。\n{}", why)),
                Ok(response) => response,
            };

            if let Some(text) = reply {
                if let Err(why) = msg.channel_id.say(&ctx.http, text).await {
                    println!("Error (say): {:?}", why);
                }
            }
        }
        Ok(())
    }

    async fn chat(&mut self, msg: &str) -> Result<Option<String>, GeminiError> {
        let mut request = self.gemini.generate_content(&self.model_name);
        request.message(msg);
        let response = request.await?;
        Ok(response.candidates[0].content.parts[0].text.clone())
    }
}
