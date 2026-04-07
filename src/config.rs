#[derive(serde::Deserialize)]
pub struct Config {
    pub models: ModelsConfig,
    pub chunking: ChunkingConfig,
    pub retrieval: RetrievalConfig,
    pub storage: StorageConfig,
}

#[derive(serde::Deserialize)]
pub struct ModelsConfig {
    pub llm_model: String,
    pub api_url: String,
    #[serde(default)]
    pub engine: Option<String>,
    pub embedding_model: String,
    #[serde(default)]
    pub chunking_model: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct ChunkingConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    #[serde(default)]
    pub smart: bool,
    #[serde(default)]
    pub smart_max_chars: Option<usize>,
}

#[derive(serde::Deserialize)]
pub struct RetrievalConfig {
    pub top_k: usize,
    pub final_k: usize,
}

#[derive(serde::Deserialize)]
pub struct StorageConfig {
    pub store_path: String,
}

pub fn load_config() -> Config {
    let config_str = std::fs::read_to_string("config.json").expect("Failed to read config.json");
    serde_json::from_str(&config_str).expect("Failed to parse config.json")
}

impl ModelsConfig {
    pub fn chat_base_url(&self) -> String {
        let engine = self.engine.as_deref().unwrap_or("llama.cpp");
        expand_url_template(
            &self.api_url,
            &self.llm_model,
            &self.embedding_model,
            &self.llm_model,
            engine,
        )
    }

    pub fn embeddings_base_url(&self) -> String {
        let engine = self.engine.as_deref().unwrap_or("llama.cpp");
        expand_url_template(
            &self.api_url,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_config_from_json() {
        let json = r#"
        {
          "models": {
            "llm_model": "ai/ministral3",
            "embedding_model": "ai/mxbai-embed-large",
            "api_url": "http://localhost:12434/engines/{engine}/v1",
            "engine": "llama.cpp",
            "chunking_model": "ai/ministral3"
          },
          "chunking": {
            "chunk_size": 500,
            "chunk_overlap": 2,
            "smart": true,
            "smart_max_chars": 4000
          },
          "retrieval": {
            "top_k": 10,
            "final_k": 8
          },
          "storage": {
            "store_path": "store.json"
          }
        }
        "#;

        let cfg: Config = serde_json::from_str(json).expect("config should parse");
        assert_eq!(cfg.models.llm_model, "ai/ministral3");
        assert_eq!(cfg.models.embedding_model, "ai/mxbai-embed-large");
        assert_eq!(cfg.models.engine.as_deref(), Some("llama.cpp"));
        assert_eq!(cfg.chunking.chunk_size, 500);
        assert!(cfg.chunking.smart);
        assert_eq!(cfg.retrieval.top_k, 10);
        assert_eq!(cfg.storage.store_path, "store.json");
    }

    #[test]
    fn expands_url_template_for_chat_and_embeddings() {
        let models = ModelsConfig {
            llm_model: "ai/ministral3".to_string(),
            embedding_model: "ai/mxbai-embed-large".to_string(),
            api_url: "http://localhost:12434/engines/{engine}/v1".to_string(),
            engine: Some("llama.cpp".to_string()),
            chunking_model: None,
        };

        assert_eq!(
            models.chat_base_url(),
            "http://localhost:12434/engines/llama.cpp/v1"
        );
        assert_eq!(
            models.embeddings_base_url(),
            "http://localhost:12434/engines/llama.cpp/v1"
        );
    }
}