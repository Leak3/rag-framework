#[derive(serde::Deserialize)]
pub struct Config {
    pub llm_model: String,
    pub llm_api_url: String,
    pub embedding_model: String,
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