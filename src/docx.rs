use docx_rs::*;

#[derive(Debug)]
enum ParsedDoc {
    Paragraph(Option<String>),
    Table(DocxTable),
}
fn parse_doc(child: DocumentChild) -> Option<ParsedDoc> {
    match child {
        DocumentChild::Paragraph(child) => {
            let list = child.clone().children;

            if list.len() > 0 {
                let resp = get_runner(list[0].clone());

                Some(ParsedDoc::Paragraph(resp))
            } else {
                None
            }
        }
        DocumentChild::Table(tb) => {
            let mut table = DocxTable::new(*tb);
            table.scan_table();
            table.scan_rows();

            Some(ParsedDoc::Table(table))
        }
        _ => None,
    }
}

fn get_runner(paragraph: ParagraphChild) -> Option<String> {
    let mut res = String::new();

    match paragraph {
        ParagraphChild::Run(rnrs) => {
            let rnr_children = rnrs.clone().children;

            if rnr_children.len() > 0 {
                let parsed = parse_runner(rnr_children[0].clone());

                match parsed {
                    Some(txt) => match txt {
                        ParsedRunner::Text(txt) => {
                            res.push_str(&txt.text);
                        }
                    },
                    None => (),
                }
            } else {
                res.push_str("\n");
            }
            Some(res)
        }
        _ => None,
    }
}

#[derive(Debug)]
enum ParsedRunner {
    Text(docx_rs::Text),
}
fn parse_runner(rnr: RunChild) -> Option<ParsedRunner> {
    match rnr {
        RunChild::Text(rnr) => Some(ParsedRunner::Text(rnr.clone())),
        _ => None,
    }
}

#[derive(Debug)]
struct Cell {
    text: String,
}
#[derive(Debug)]
struct Row {
    cells: Vec<Cell>,
    raw: TableRow,
}
#[derive(Debug)]
struct DocxTable {
    rows: Vec<Row>,
    raw: Table,
}
impl DocxTable {
    pub fn new(table: Table) -> Self {
        Self {
            rows: vec![],
            raw: table,
        }
    }
    pub fn scan_table(&mut self) {
        let rows = self.raw.rows.clone();

        for t_child in rows {
            match t_child {
                TableChild::TableRow(r) => {
                    self.rows.push(Row {
                        cells: vec![],
                        raw: r.clone(),
                    });
                }
            };
        }
    }
    pub fn scan_rows(&mut self) {
        for r in self.rows.iter_mut() {
            for r_child in r.raw.cells.clone() {
                let mut cell = Cell {
                    text: String::new(),
                };
                match r_child {
                    TableRowChild::TableCell(c) => {
                        let content = c.children;
                        for co in content {
                            match co {
                                TableCellContent::Paragraph(p) => {
                                    let list = p.children;
                                    if list.len() > 0 {
                                        let rnr = get_runner(list[0].clone());

                                        match rnr {
                                            Some(r) => cell.text = r,
                                            _ => (),
                                        }
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                };
                r.cells.push(cell);
            }
        }
    }
    pub fn convert(&self) -> String {
        let mut res = String::new();

        for r in self.rows.iter() {
            for c in r.cells.iter() {
                let formated = &format!(" {}", c.text);
                res.push_str(&formated.as_str());
            }
        }

        res
    }
}

pub fn get(file_path: &str) -> Result<String, ()> {
    let file = std::fs::read(file_path).unwrap();
    let document = read_docx(file.as_slice()).unwrap().document;
    let mut res = String::new();

    for doc_child in document.children {
        let parsed_doc = parse_doc(doc_child);

        match parsed_doc {
            Some(dc) => match dc {
                ParsedDoc::Paragraph(p) => match p {
                    Some(txt) => {
                        let formatted = &format!("{}\n", &txt);
                        res.push_str(&formatted);
                    }
                    _ => (),
                },
                ParsedDoc::Table(t) => {
                    res.push_str(&t.convert());
                    ()
                }
            },
            _ => (),
        }
    }
    Ok(res)
}
