use crate::models::general::llm::{APIResponse, ChatCompletion, Message};
use dotenv::dotenv;
use reqwest::Client;
use std::env;

use reqwest::header::{HeaderMap, HeaderValue};

pub async fn call_gpt(messages: Vec<Message>) -> Result<String, Box<dyn std::error::Error + Send>> {
    dotenv().ok();

    // Extract api information
    let api_key: String =
        env::var("OPEN_AI_KEY").expect("OPEN_AI_KEY not found in environment variables");
    let api_org: String =
        env::var("OPEN_AI_ORG").expect("OPEN_AI_ORG not found in environment variables");
    let api_model: String =
        env::var("OPEN_AI_MODEL").expect("OPEN_AI_MODEL not found in environment variables");

    // confirm endpoint
    let url: &str = "https://api.openai.com/v1/chat/completions";

    // create headers
    let mut headers: HeaderMap = HeaderMap::new();
    // create api key header
    headers.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?,
    );
    // create api org header
    headers.insert(
        "OpenAI-Organization",
        HeaderValue::from_str(api_org.as_str())
            .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?,
    );

    // create client
    let client = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

    let chat_completion: ChatCompletion = ChatCompletion {
        model: api_model.to_string(),
        messages,
        temperature: 0.1,
    };

    // extract api response
    let res: APIResponse = client
        .post(url)
        .json(&chat_completion)
        .send()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?
        .json()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

    // send response
    Ok(res.choices[0].message.content.clone())
}
#[cfg(test)]
mod tests {

    use super::*;
    #[tokio::test]
    async fn tests_call_to_openai() {
        let message = Message {
            role: "user".to_string(),
            content: "Hi there, this is a test. Give me a short response.".to_string(),
        };
        let messages: Vec<Message> = vec![message];
        let res: Result<String, Box<dyn std::error::Error + Send>> = call_gpt(messages).await;
        if let Ok(res_str) = res {
            dbg!(res_str);
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
