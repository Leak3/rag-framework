#[derive(serde::Serialize)]
struct EmbedRequest {
    model: String,
    input: Vec<String>,
}

#[derive(serde::Deserialize)]
struct EmbedResponse {
    data: Vec<EmbedData>,
}

#[derive(serde::Deserialize)]
struct EmbedData {
    embedding: Vec<f32>,
}

pub async fn embed(config: &crate::config::Config, text: &str) -> Result<Vec<f32>, crate::error::Error> {
    let client = reqwest::Client::new();
    let base_url = config.models.embeddings_base_url();
    let request_body = EmbedRequest {
        model: config.models.embedding_model.clone(),
        input: vec![text.to_string()],
    };

    let response = client
        .post(format!("{}/embeddings", base_url))
        .json(&request_body)
        .send()
        .await?;

    let embed_response: EmbedResponse = response.json().await?;
    embed_response
        .data
        .into_iter()
        .next()
        .map(|d| d.embedding)
        .ok_or_else(|| crate::error::Error::new(500, "empty embeddings response"))
}
