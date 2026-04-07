#[derive(serde::Deserialize)]
pub struct Config {
    pub llm_model: String,
    pub llm_api_url: String,
    #[serde(default)]
    pub llm_engine: Option<String>,
    pub embedding_model: String,
    #[serde(default)]
    pub embedding_engine: Option<String>,
    pub store_path: String,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub top_k: usize,
    pub final_k: usize,
}

pub fn load_config() -> Config {
    let config_str = std::fs::read_to_string("config.json").expect("Failed to read config.json");
    serde_json::from_str(&config_str).expect("Failed to parse config.json")
}

impl Config {
    pub fn chat_base_url(&self) -> String {
        let engine = self.llm_engine.as_deref().unwrap_or("llama.cpp");
        expand_url_template(
            &self.llm_api_url,
            &self.llm_model,
            &self.embedding_model,
            &self.llm_model,
            engine,
        )
    }

    pub fn embeddings_base_url(&self) -> String {
        let engine = self.embedding_engine.as_deref().unwrap_or_else(|| {
            self.llm_engine
                .as_deref()
                .unwrap_or("llama.cpp")
        });
        expand_url_template(
            &self.llm_api_url,
            &self.llm_model,
            &self.embedding_model,
            &self.embedding_model,
            engine,
        )
    }
}

fn expand_url_template(
    template: &str,
    llm_model: &str,
    embedding_model: &str,
    model: &str,
    engine: &str,
) -> String {
    template
        .replace("{llm_model}", llm_model)
        .replace("{embedding_model}", embedding_model)
        .replace("{model}", model)
        .replace("{engine}", engine)
        .trim_end_matches('/')
        .to_string()
}