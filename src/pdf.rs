use lopdf::Document;

pub fn get(file_path: &str) -> Result<String, ()> {
    let doc = Document::load(file_path).unwrap();
    let pages = doc.get_pages();
    let mut res = String::new();
    for page in pages {
        let p = [page.0];
        let text = doc.extract_text(&p).unwrap();

        res.push_str(text.as_str());
    }

    Ok(res)
}
