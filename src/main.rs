use crawler::analyze;
mod db;
use db::Db;
use strsim::{levenshtein, normalized_levenshtein};

mod crawler;
mod docx;
mod pdf;

fn scan(db: &Db, path: &str) {
    let (dirs, files) = analyze(path);

    for dir in dirs {
        db.insert_dir(&dir.name, &dir.path);

        println!("adding DIR :  {}", dir.name);
        scan(db, &dir.path);
    }
    for file in files {
        let content = match file.file_type.as_str() {
            "pdf" => pdf::get(&file.path).unwrap(),
            "docx" => docx::get(&file.path).unwrap(),
            _ => "".to_string(),
        };

        println!("adding FILE: {}", file.file_name);
        db.insert_file(&file.file_name, &file.file_type, &file.path, &content);
    }
}
fn main() {
    // DEFAULT FLOW
    // let args: Vec<String> = std::env::args().collect();
    // let path = args[1].clone();
    // let db = Db::new();
    // scan(&db, path.as_str());


    // SEARCHING
    // let resp = normalized_levenshtein("soeifwkgkwg o5hrgok qwerty fokrgo ohom4hmkokm", "qwerty");
    // println!("{resp:?}");

    let q = docx::get("./dir/more/Основные_положения_ГОСТ_21_508_93_ПРАВИЛА_ВЫП_ГЕНПЛАНОВ.docx").unwrap();
    println!("{q:?}");
}
