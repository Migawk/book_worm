use crate::db::{DbFileWithoutContent, Dir};
use std::fs;

fn get_type(file_name: String) -> String {
    let collection: Vec<&str> = file_name.split(".").collect();
    let idx = collection.len() - 1;
    String::from(collection[idx])
}
pub fn analyze(path: &str) -> (Vec<Dir>, Vec<DbFileWithoutContent>) {
    let dir = fs::read_dir(path).unwrap();

    let mut dirs: Vec<Dir> = vec![];
    let mut files: Vec<DbFileWithoutContent> = vec![];
    for entry in dir {
        let ent = entry.unwrap();
        let file_name = ent.file_name().to_str().unwrap().to_string();
        let file_type = get_type(file_name.clone());
        let path = ent.path().to_str().unwrap().to_string();
        let is_dir = ent.file_type().unwrap().is_dir();

        if is_dir {
            let dir = Dir {
                name: file_name,
                path,
            };
            dirs.push(dir);
        } else {
            let file = DbFileWithoutContent {
                file_name,
                file_type,
                path,
            };

            files.push(file);
        }
    }

    (dirs, files)
}
