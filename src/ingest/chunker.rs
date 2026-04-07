pub fn chunk_text(text: &str, chunk_size: usize, overlap_sentences: usize) -> Vec<String> {
    let sentences: Vec<&str> = text.split_inclusive(|c| c == '.' || c == '!' || c == '?').collect();
    let mut current_chunk: Vec<&str> = Vec::new();
    let mut chunks: Vec<String> = Vec::new();

    for sentence in &sentences {
        current_chunk.push(sentence);

        let current_len: usize = current_chunk.iter().map(|s| s.len()).sum();

        if current_len >= chunk_size {
            chunks.push(current_chunk.join(""));
            current_chunk = current_chunk.split_off(current_chunk.len().saturating_sub(overlap_sentences));
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk.join(""));
    }

    chunks
}
