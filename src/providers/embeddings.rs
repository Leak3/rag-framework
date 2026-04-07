#[derive(serde::Serialize)]
struct EmbedRequest {
    model: String,
    input: String,
}

#[derive(serde::Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

pub async fn embed(text: &str) -> Result<Vec<f32>, crate::error::Error> {
    let client = reqwest::Client::new();
    let request_body = EmbedRequest {
        model: "nomic-embed-text".to_string(),
        input: text.to_string(),
    };

    let response = client
        .post("http://localhost:11434/api/embed")
        .json(&request_body)
        .send()
        .await?;

    let embed_response: EmbedResponse = response.json().await?;
    embed_response.embeddings.into_iter().next()
        .ok_or_else(|| crate::error::Error::new(500, "empty embeddings response"))
}
