use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub source: String,
    pub text: String,
    pub embedding: Vec<f32>,
    pub term_frequencies: HashMap<String, usize>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct VectorStore {
    documents: Vec<Document>,
    doc_frequencies: HashMap<String, usize>,
}

impl VectorStore {
    pub fn new() -> Self {
        VectorStore { documents: Vec::new(), doc_frequencies: HashMap::new() }
    }

    pub fn add(&mut self, text: String, embedding: Vec<f32>, source: &str) {
        let term_frequencies = tokenize(&text);
        self.documents.push(Document { text, embedding, term_frequencies, source: source.to_string() });
        self.generate_doc_frequencies();
    }

    pub fn delete_by_source(&mut self, source: &str) {
        self.documents.retain(|doc| doc.source != source);
        self.generate_doc_frequencies();
    }

    pub fn vector_search(&self, query_embedding: &[f32], top_k: usize) -> Vec<String> {
        fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
            let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
            let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
            let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
            dot / (mag_a * mag_b)
        }

        let mut results: Vec<(f32, &Document)> = self.documents.iter()
            .map(|doc| (cosine_similarity(query_embedding, &doc.embedding), doc))
            .collect();

        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        results.truncate(top_k);

        results.into_iter().map(|(_, doc)| doc.text.clone()).collect()
    }

    pub fn bm25_search(&self, query: &str, top_k: usize) -> Vec<String> {
        let query_terms = tokenize(query);
        let mut scores: Vec<(f32, &Document)> = self.documents.iter()
            .map(|doc| {
                let mut score = 0.0;
                for (term, q_freq) in &query_terms {
                    let d_freq = *doc.term_frequencies.get(term).unwrap_or(&0) as f32;
                    let n = self.documents.len() as f32;
                    let df = *self.doc_frequencies.get(term).unwrap_or(&1) as f32;
                    let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
                    score += idf * (d_freq * *q_freq as f32) / (d_freq + 1.5);
                }
                (score, doc)
            })
            .collect();

        scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        scores.truncate(top_k);

        scores.into_iter().map(|(_, doc)| doc.text.clone()).collect()
    }

    pub fn save(&self, path: &str) {
        let json =serde_json::to_string(self).unwrap(); 
        std::fs::write(path, json).unwrap();
    }

    pub fn load(path: &str) -> Self {
        if !std::path::Path::new(path).exists() {
            return VectorStore::new();
        }

        let json = std::fs::read_to_string(path).unwrap();
        let store: VectorStore = serde_json::from_str(&json).unwrap();
        store
    }

    fn generate_doc_frequencies(&mut self) {
        self.doc_frequencies.clear();
        for doc in &self.documents {
            for term in doc.term_frequencies.keys() {
                *self.doc_frequencies.entry(term.clone()).or_insert(0) += 1;
            }
        }
    }
}

fn tokenize(text: &str) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    let scrubbed = text.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect::<String>();
    let words = scrubbed.split_whitespace().collect::<Vec<&str>>();

    for word in words {
        *map.entry(word.to_string().to_lowercase()).or_insert(0) += 1;
    }

    map
}
