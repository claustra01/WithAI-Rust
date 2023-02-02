#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate line_bot_sdk_rust as line;

use std::env;
use dotenv::dotenv;
use rocket::http::Status;
use futures::executor::block_on;

use line::bot::LineBot;
use line::events::messages::MessageType as EventMessageType;
use line::events::{EventType, Events};
use line::messages::{SendMessageType, TextMessage};
use line::support::rocket_support::{Body, Signature};

use openai_api_rs::v1::completion::{self, CompletionRequest};
use openai_api_rs::v1::api::Client;

async fn openai(question: String) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let req = CompletionRequest {
        model: completion::GPT3_TEXT_DAVINCI_003.to_string(),
        prompt: Some(question),
        suffix: None,
        max_tokens: Some(3000),
        temperature: Some(0.9),
        top_p: Some(1.0),
        n: None,
        stream: None,
        logprobs: None,
        echo: None,
        stop: None,
        presence_penalty: Some(0.6),
        frequency_penalty: Some(0.0),
        best_of: None,
        logit_bias: None,
        user: None,
      };
    let result = client.completion(req).await?;
    let answer = String::from(&result.choices[0].text);

    Ok(answer)
}

#[post("/callback", data = "<body>")]
fn callback(signature: Signature, body: Body) -> Status {
    // Get channel secret and access token by environment variable
    let channel_secret: &str =
        &env::var("LINE_CHANNEL_SECRET").expect("Failed getting LINE_CHANNEL_SECRET");
    let access_token: &str =
        &env::var("LINE_CHANNEL_ACCESS_TOKEN").expect("Failed getting LINE_CHANNEL_ACCESS_TOKEN");

    // LineBot
    let bot = LineBot::new(channel_secret, access_token);

    // Request body parse
    let result: Result<Events, &'static str> =
        bot.parse_event_request(&signature.key, &body.string);

    // Success parsing
    if let Ok(res) = result {
        for event in res.events {
            // MessageEvent only
            if let EventType::MessageEvent(message_event) = event.r#type {
                // TextMessageEvent only
                if let EventMessageType::TextMessage(text_message) = message_event.message.r#type {
                    // Get answer by OpenAI
                    let answer = block_on(openai(text_message.text)).unwrap();
                    // Create TextMessage
                    let message = SendMessageType::TextMessage(TextMessage {
                        text: answer,
                        emojis: None,
                    });
                    // Reply message with reply_token
                    let _res = bot.reply_message(&message_event.reply_token, vec![message]);
                }
            }
        }
        return Status::new(200, "OK");
    }
    // Failed parsing
    else if let Err(msg) = result {
        return Status::new(500, msg);
    }
    Status::new(500, "Internal Server Error")
}

fn main() {
    dotenv().ok();
    rocket::ignite().mount("/", routes![callback]).launch();
}