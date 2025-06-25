use std::error::Error;

use futures::channel::mpsc::UnboundedReceiver;

use futures::StreamExt;
use gemini_rs::{
    Client, Error as GeminiError,
    types::{
        Content, HarmBlockThreshold,
        HarmCategory::{self, *},
        Part, Role, SafetySettings,
    },
};
use serenity::{client::Context, model::channel::Message};

type Receiver = UnboundedReceiver<(Context, Message)>;

const HARM_CATEGORIES: [HarmCategory; 5] = [
    HarmCategoryHarassment,
    HarmCategoryHateSpeech,
    HarmCategorySexuallyExplicit,
    HarmCategoryDangerousContent,
    HarmCategoryCivicIntegrity,
];

pub struct Risajuu {
    receiver: Receiver,
    gemini: Client,
    model_name: String,
    history: Vec<Content>,
    system_instruction: String,
    safety_settings: Vec<SafetySettings>,
}

impl Risajuu {
    pub fn new(api_key: &str, receiver: Receiver, model_name: String, system_instruction: String) -> Risajuu {
        let gemini = gemini_rs::Client::new(api_key);
        let history = Vec::new();
        let mut safety_settings = Vec::new();
        for hc in HARM_CATEGORIES {
            let ss = SafetySettings {
                category: hc,
                threshold: HarmBlockThreshold::BlockNone,
            };
            safety_settings.push(ss);
        }
        Risajuu {
            receiver,
            gemini,
            model_name,
            history,
            system_instruction,
            safety_settings,
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
                let mut iter = text.split('\n').filter(|line| !line.is_empty());
                while let Some(chunk) = iter.next() {
                    if let Err(why) = msg.channel_id.say(&ctx.http, chunk).await {
                        println!("Error (say): {:?}", why);
                    }
                }
            }
        }
        Ok(())
    }

    async fn chat(&mut self, msg: &str) -> Result<Option<String>, GeminiError> {
        let mut request = self.gemini.generate_content(&self.model_name);
        request.safety_settings(self.safety_settings.clone());
        request.system_instruction(&self.system_instruction);
        request.contents(self.history.clone());
        request.message(msg);
        let response = request.await?;
        let response = response.candidates[0].content.parts[0].text.clone();
        let user_content = Content {
            role: Role::User,
            parts: vec![Part::text(msg)],
        };
        self.history.push(user_content);
        if let Some(text) = &response {
            let model_content = Content {
                role: Role::Model,
                parts: vec![Part::text(text)],
            };
            self.history.push(model_content);
        };
        Ok(response)
    }
}
