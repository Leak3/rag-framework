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

async fn chat(
    config: &crate::config::Config,
    model: &str,
    messages: Vec<Message>,
) -> Result<String, crate::error::Error> {
    let client = reqwest::Client::new();
    let base_url = config.models.chat_base_url();

    let response = client
        .post(format!("{}/chat/completions", base_url))
        .json(&ChatRequest {
            model: model.to_string(),
            messages,
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

pub async fn ask(config: &crate::config::Config, prompt: String) -> Result<String, crate::error::Error> {
    chat(
        config,
        &config.models.llm_model,
        vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
    )
    .await
}

pub async fn ask_with_model(
    config: &crate::config::Config,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, crate::error::Error> {
    chat(
        config,
        model,
        vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ],
    )
    .await
}
