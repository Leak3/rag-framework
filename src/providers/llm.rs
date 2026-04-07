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

pub async fn ask(prompt: String) -> Result<String, crate::error::Error> {
    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&ChatRequest {
            model: "llama3.1:8b".to_string(),
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
