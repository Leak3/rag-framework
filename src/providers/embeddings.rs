#[derive(serde::Serialize)]
struct EmbedRequest {
    model: String,
    input: String,
}

#[derive(serde::Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

pub async fn embed(config: &crate::config::Config, text: &str) -> Result<Vec<f32>, crate::error::Error> {
    let client = reqwest::Client::new();
    let base_url = config.llm_api_url.trim_end_matches('/');
    let request_body = EmbedRequest {
        model: config.embedding_model.clone(),
        input: text.to_string(),
    };

    let response = client
        .post(format!("{}/api/embed", base_url))
        .json(&request_body)
        .send()
        .await?;

    let embed_response: EmbedResponse = response.json().await?;
    embed_response.embeddings.into_iter().next()
        .ok_or_else(|| crate::error::Error::new(500, "empty embeddings response"))
}
