#[derive(serde::Serialize, serde::Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(serde::Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(serde::Deserialize)]
struct ChatResponse {
    message: Message,
}

pub async fn ask(config: &crate::config::Config, prompt: String) -> Result<String, crate::error::Error> {
    let client = reqwest::Client::new();
    let base_url = config.llm_api_url.trim_end_matches('/');

    let response = client
        .post(format!("{}/api/chat", base_url))
        .json(&ChatRequest {
            model: config.llm_model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            stream: false,
        })
        .send()
        .await?;

    let chat_response: ChatResponse = response.json().await?;
    Ok(chat_response.message.content)
}
