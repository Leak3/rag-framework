#[derive(serde::Deserialize)]
struct SmartChunkResponse {
    chunks: Vec<SmartChunk>,
}

#[derive(serde::Deserialize)]
struct SmartChunk {
    start: usize,
    end: usize,
}

pub fn chunk_by_sentences(text: &str, chunk_size: usize, overlap_sentences: usize) -> Vec<String> {
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

pub async fn chunk_document(
    config: &crate::config::Config,
    text: &str,
) -> Vec<String> {
    let fallback = || chunk_by_sentences(text, config.chunking.chunk_size, config.chunking.chunk_overlap);

    if !config.chunking.smart {
        println!("[chunking] smart_chunking disabled; using programmatic chunking");
        return fallback();
    }
    if !has_semantic_boundaries(text) {
        println!("[chunking] no clear boundaries detected; using programmatic chunking");
        return fallback();
    }

    let max_chars = config.chunking.smart_max_chars.unwrap_or(4000);
    let model = config
        .models
        .chunking_model
        .as_deref()
        .unwrap_or(&config.models.llm_model);

    println!(
        "[chunking] attempting smart chunking (model={}, max_chars={})",
        model, max_chars
    );

    let system = "You split documents into retrieval chunks.\n\
Return ONLY valid JSON and do not wrap it in markdown.\n\
Rules:\n\
- Output schema exactly: {\"chunks\":[{\"start\":0,\"end\":123}]}\n\
- \"start\" and \"end\" are CHARACTER indices into the input text (0-based, end is exclusive).\n\
- Chunks must cover the entire input contiguously: first chunk start=0, each next chunk start equals previous end, last chunk end equals total character count.\n\
- Prefer splitting on headings, paragraph breaks, and list boundaries.\n\
- Chunks must be contiguous and there should be no overlap between chunks.\n\
- Each chunk must be <= MAX_CHARS characters.\n\
- If the document has no clear boundaries, return {\"chunks\":[]}.\n";

    let user = format!(
        "MAX_CHARS={}\n\nINPUT:\n<<<\n{}\n>>>",
        max_chars, text
    );

    let llm_out_1 = match crate::providers::llm::ask_with_model(config, model, system, &user).await {
        Ok(s) => s,
        Err(e) => {
            println!("[chunking] smart chunking failed (llm error: {:?}); falling back", e);
            return fallback();
        }
    };

    let parsed: SmartChunkResponse = match extract_json_object(&llm_out_1).and_then(|json| {
        serde_json::from_str(&json).ok()
    }) {
        Some(p) => p,
        None => {
            println!(
                "[chunking] smart chunking invalid JSON (attempt 1). model_output_snippet=\"{}\"",
                truncate_for_log(&llm_out_1, 220)
            );

            let system_retry = "Return ONLY valid JSON. No explanation, no markdown.\n\
Output schema exactly: {\"chunks\":[{\"start\":0,\"end\":123}]}\n\
\"start\" and \"end\" are CHARACTER indices (0-based, end exclusive).\n\
Chunks must cover the input contiguously (no gaps/overlap) and cover the full document.\n\
Your response MUST start with '{' and end with '}'.\n";

            let llm_out_2 =
                match crate::providers::llm::ask_with_model(config, model, system_retry, &user).await {
                    Ok(s) => s,
                    Err(e) => {
                        println!(
                            "[chunking] smart chunking retry failed (llm error: {:?}); falling back",
                            e
                        );
                        return fallback();
                    }
                };

            match extract_json_object(&llm_out_2).and_then(|json| serde_json::from_str(&json).ok())
            {
                Some(p) => p,
                None => {
                    println!(
                        "[chunking] smart chunking invalid JSON (attempt 2). model_output_snippet=\"{}\"; falling back",
                        truncate_for_log(&llm_out_2, 220)
                    );
                    return fallback();
                }
            }
        }
    };

    let chunks: Vec<String> = match apply_smart_chunk_ranges(text, &parsed.chunks, max_chars) {
        Ok(c) => c,
        Err(reason) => {
            println!(
                "[chunking] smart chunking output rejected by validation ({}); falling back",
                reason
            );
            return fallback();
        }
    };

    if !validate_smart_chunks(text, &chunks, max_chars) {
        println!("[chunking] smart chunking output rejected by validation; falling back");
        return fallback();
    }
    println!("[chunking] smart chunking succeeded ({} chunks)", chunks.len());
    chunks
}

fn has_semantic_boundaries(text: &str) -> bool {
    let newline_count = text.matches('\n').count();
    if newline_count < 8 {
        return false;
    }
    let has_heading = text.contains("\n#")
        || text.contains("\n##")
        || text.contains("\n###")
        || text.contains("\n====")
        || text.contains("\n----");
    let has_lists = text.contains("\n- ")
        || text.contains("\n* ")
        || text.contains("\n1. ")
        || text.contains("\n2. ");
    has_heading || has_lists
}

fn truncate_for_log(s: &str, max_len: usize) -> String {
    let mut t = s
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', " ");
    t = t.trim().to_string();
    if t.len() > max_len {
        t.truncate(max_len);
        t.push_str("…");
    }
    t
}

fn extract_json_object(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Strip common markdown fences.
    let mut t = trimmed;
    if let Some(rest) = t.strip_prefix("```json") {
        t = rest;
    } else if let Some(rest) = t.strip_prefix("```") {
        t = rest;
    }
    t = t.trim();
    if let Some(rest) = t.strip_suffix("```") {
        t = rest.trim();
    }

    // If the model added leading text, take the first JSON object-looking span.
    let start = t.find('{')?;
    let end = t.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(t[start..=end].to_string())
}

fn validate_smart_chunks(original: &str, chunks: &[String], max_chars: usize) -> bool {
    if chunks.is_empty() {
        return false;
    }
    // Reject "single giant chunk" or oversize chunks.
    if chunks.len() == 1 && chunks[0].len() > max_chars {
        return false;
    }
    if chunks.iter().any(|c| c.is_empty() || c.len() > max_chars) {
        return false;
    }

    // Ensure most chunk text exists in original (prevents hallucinated content).
    let mut contained = 0usize;
    for c in chunks {
        if original.contains(c) {
            contained += 1;
        }
    }
    contained * 10 >= chunks.len() * 9
}

fn char_to_byte_index(s: &str, char_index: usize) -> Option<usize> {
    if char_index == 0 {
        return Some(0);
    }
    s.char_indices()
        .nth(char_index)
        .map(|(byte_idx, _)| byte_idx)
        .or_else(|| {
            // Allow end == total chars.
            if char_index == s.chars().count() {
                Some(s.len())
            } else {
                None
            }
        })
}

fn apply_smart_chunk_ranges(text: &str, ranges: &[SmartChunk], max_chars: usize) -> Result<Vec<String>, &'static str> {
    if ranges.is_empty() {
        return Err("empty ranges");
    }

    let mut out = Vec::with_capacity(ranges.len());
    let mut last_end = 0usize;
    let total_chars = text.chars().count();

    for r in ranges {
        if r.start >= r.end {
            return Err("invalid range (start>=end)");
        }
        if r.start < last_end {
            return Err("overlapping ranges");
        }
        if r.start > last_end {
            // Allow gaps only if they are whitespace.
            let gap_start_b = char_to_byte_index(text, last_end).ok_or("gap index invalid")?;
            let gap_end_b = char_to_byte_index(text, r.start).ok_or("gap index invalid")?;
            let gap = &text[gap_start_b..gap_end_b];
            if !gap.trim().is_empty() {
                return Err("non-whitespace gap between ranges");
            }
            // Treat whitespace gap as part of the next chunk (by shifting start back).
            // This preserves full coverage without needing the model to count whitespace perfectly.
        }
        if r.end - r.start > max_chars {
            return Err("range exceeds max_chars");
        }

        let start_b = char_to_byte_index(text, last_end).ok_or("start index invalid")?;
        let end_b = char_to_byte_index(text, r.end).ok_or("end index invalid")?;
        if start_b > end_b || end_b > text.len() {
            return Err("byte indices invalid");
        }
        let chunk = text[start_b..end_b].to_string();
        if chunk.trim().is_empty() {
            return Err("empty chunk");
        }
        out.push(chunk);
        last_end = r.end;
    }

    // Ensure coverage reaches end of document (allow trailing whitespace, or append tail if it fits).
    if last_end < total_chars {
        let tail_start_b = char_to_byte_index(text, last_end).ok_or("tail index invalid")?;
        let tail = &text[tail_start_b..];
        if !tail.trim().is_empty() {
            let tail_chars = total_chars - last_end;
            if tail_chars <= max_chars {
                out.push(tail.to_string());
                last_end = total_chars;
            } else {
                return Err("tail not covered and too large to append");
            }
        } else {
            last_end = total_chars;
        }
    }

    if last_end != total_chars {
        return Err("did not reach end of document");
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_allow_whitespace_gaps() {
        let text = "A\n\nB";
        // Gap is two newlines (whitespace). Model might skip them.
        let ranges = vec![SmartChunk { start: 0, end: 1 }, SmartChunk { start: 3, end: 4 }];
        let chunks = apply_smart_chunk_ranges(text, &ranges, 10).expect("should accept whitespace gap");
        assert_eq!(chunks, vec!["A".to_string(), "\n\nB".to_string()]);
    }

    #[test]
    fn ranges_reject_non_whitespace_gaps() {
        let text = "A x B";
        // Gap includes " x " (non-whitespace after trimming), should be rejected.
        let ranges = vec![SmartChunk { start: 0, end: 1 }, SmartChunk { start: 4, end: 5 }];
        let err = apply_smart_chunk_ranges(text, &ranges, 10).unwrap_err();
        assert_eq!(err, "non-whitespace gap between ranges");
    }

    #[test]
    fn ranges_append_small_tail_if_missing() {
        let text = "Hello\nWorld";
        // Only covers "Hello" (0..5), tail is "\nWorld" and should be appended.
        let ranges = vec![SmartChunk { start: 0, end: 5 }];
        let chunks = apply_smart_chunk_ranges(text, &ranges, 4000).expect("should append tail");
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "Hello");
        assert_eq!(chunks[1], "\nWorld");
    }
}
