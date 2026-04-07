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
    choices: Vec<Choice>,
}

#[derive(serde::Deserialize)]
struct Choice {
    message: Message,
}

pub async fn ask(config: &crate::config::Config, prompt: String) -> Result<String, crate::error::Error> {
    let client = reqwest::Client::new();
    let base_url = config.chat_base_url();

    let response = client
        .post(format!("{}/chat/completions", base_url))
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
    chat_response
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| crate::error::Error::new(500, "empty chat response"))
}
