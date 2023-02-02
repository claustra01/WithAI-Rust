use openai_api_rs::v1::completion::{self, CompletionRequest};
use openai_api_rs::v1::api::Client;
use std::env;
use dotenv::dotenv;
use futures::executor::block_on;

async fn openai(question: String) -> Result<String, Box<dyn std::error::Error>> {
    dotenv().ok();
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

#[tokio::main]
async fn main() {
    let mut question = String::new();
    std::io::stdin().read_line(&mut question);
    let answer = block_on(openai(question));
    println!("{:?}", answer);
}