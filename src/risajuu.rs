use std::error::Error;

use futures::{Stream, channel::mpsc::UnboundedReceiver};

use futures::StreamExt;
use gemini_rs::types::Response;
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
            let mut stream = self.chat(&msg.content).await?;
            while let Some(chunk) = stream.next().await {
                let reply = match chunk {
                    Ok(res) => res.candidates[0].content.parts[0].text.clone(),
                    Err(why) => Some(format!("Geminiのエラーじゅう。\n{}", why)),
                };
                println!("{:?}", reply);
                if let Some(text) = reply {
                    if let Err(why) = msg.channel_id.say(&ctx.http, text).await {
                        println!("Error (say): {:?}", why);
                    }
                }
            }
        }
        Ok(())
    }

    async fn chat(&mut self, msg: &str) -> Result<impl Stream<Item = Result<Response, GeminiError>>, Box<dyn Error>> {
        let mut request = self.gemini.stream_generate_content(&self.model_name);
        request.safety_settings(self.safety_settings.clone());
        request.system_instruction(&self.system_instruction);
        request.contents(self.history.clone());
        request.message(msg);
        let stream = request.stream().await?;
        let user_content = Content {
            role: Role::User,
            parts: vec![Part::text(msg)],
        };
        self.history.push(user_content);

        let stream = stream.map(|chunk| {
            match &chunk {
                Ok(response) => {
                    if let Some(text) = &response.candidates[0].content.parts[0].text {
                        let model_content = Content {
                            role: Role::Model,
                            parts: vec![Part::text(text)],
                        };
                        self.history.push(model_content);
                    };
                }
                Err(why) => eprintln!("Error (stream): {:?}", why),
            };
            chunk
        });

        Ok(stream)
    }
}
