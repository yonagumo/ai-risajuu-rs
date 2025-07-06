use futures::{Stream, StreamExt};
use std::error::Error;

use gemini_rs::{
    Client, Error as GeminiError,
    types::{
        Content, HarmBlockThreshold,
        HarmCategory::{self, *},
        Part, Response, Role, SafetySettings,
    },
};

const HARM_CATEGORIES: [HarmCategory; 5] = [
    HarmCategoryHarassment,
    HarmCategoryHateSpeech,
    HarmCategorySexuallyExplicit,
    HarmCategoryDangerousContent,
    HarmCategoryCivicIntegrity,
];

#[derive(Debug, Clone)]
pub struct ChatSettings {
    pub api_key: String,
    pub model_name: String,
    pub system_instruction: String,
}

pub struct Chat {
    gemini: Client,
    model_name: String,
    history: Vec<Content>,
    system_instruction: String,
    safety_settings: Vec<SafetySettings>,
}

impl Chat {
    pub fn new(settings: ChatSettings) -> Chat {
        let mut safety_settings = Vec::new();
        for hc in HARM_CATEGORIES {
            let ss = SafetySettings {
                category: hc,
                threshold: HarmBlockThreshold::BlockNone,
            };
            safety_settings.push(ss);
        }
        Chat {
            gemini: Client::new(settings.api_key),
            model_name: settings.model_name,
            history: Vec::new(),
            system_instruction: settings.system_instruction,
            safety_settings,
        }
    }

    pub fn reset(&mut self) {
        self.history.clear();
    }

    pub async fn send_message(&mut self, msg: &str) -> Result<impl Stream<Item = Result<Response, String>>, Box<dyn Error>> {
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

        let stream = stream.map(|chunk| match chunk {
            Ok(response) => {
                let model_content = response.candidates[0].content.clone();
                self.history.push(model_content);
                Ok(response)
            }
            Err(why) => {
                eprintln!("Error (stream): {:?}", why);
                if let GeminiError::Serde(why) = &why {
                    eprintln!("Category: {:?}", why.classify());
                };
                let err = format!("Geminiライブラリのエラーじゅう。\n{}", why);
                let model_content = Content {
                    role: Role::Model,
                    parts: vec![Part::text(&err)],
                };
                self.history.push(model_content);
                Err(err)
            }
        });

        Ok(stream)
    }
}
