pub async fn extract_text(data: Vec<u8>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let result = tokio::task::spawn_blocking(move || {
        let doc = lopdf::Document::load_mem(&data)?;
        let mut text = String::new();

        for (i, _page_id) in doc.page_iter().enumerate() {
            if let Ok(page_text) = doc.extract_text(&[(i + 1) as u32]) {
                text.push_str(&page_text);
                text.push('\n');
            }
        }

        Ok::<String, lopdf::Error>(text)
    }).await?;

    Ok(result?)
}
