use docx_rs::*;

fn get_paragraph(child: DocumentChild) -> Option<Paragraph> {
    match child {
        DocumentChild::Paragraph(child) => Some(*child.clone()),
        _ => None,
    }
}
fn get_runner(paragraph: ParagraphChild) -> Option<Run> {
    match paragraph {
        ParagraphChild::Run(paragraph) => Some(*paragraph.clone()),
        _ => None,
    }
}

fn get_text(rnr: RunChild) -> Option<Text> {
    match rnr {
        RunChild::Text(rnr) => Some(rnr.clone()),
        _ => None,
    }
}

pub fn get(file_path: &str) -> Result<String, ()> {
    let file = std::fs::read(file_path).unwrap();
    let document = read_docx(file.as_slice()).unwrap().document;
    let mut res = String::new();

    for doc_child in document.children {
        let par = get_paragraph(doc_child).unwrap().children;
        for par_child in par {
            let rnr = get_runner(par_child).unwrap().children;

            println!("{:?}", rnr);
            if rnr.len() > 0 {
                let text = get_text(rnr[0].clone()).unwrap().text;
                res.push_str(&text);
            } else {
                res.push_str("\n");
            }
        }
    }
    Ok(res)
}
