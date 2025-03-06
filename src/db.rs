use std::env;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use iced::futures::io;
use regex::Regex;
use sqlite::{Connection, Value};
use strsim::{jaro, normalized_levenshtein};

use crate::{crawler, docx, pdf};
use crawler::analyze;

#[derive(Debug)]
pub struct Dir {
    pub name: String,
    pub path: String,
}
#[derive(Debug)]
pub struct DbFile {
    pub file_name: String,
    pub file_type: String,
    pub path: String,
    pub content: String,
    pub id: i64,
}

#[derive(Debug, Clone)]
pub struct DictWord {
    pub content: String,
    pub file_idx: i16,
    pub word_idx: i32,
    pub similarity: f32,
    pub file_name: String,
    pub file_path: String,
    pub file_content: String,
}
pub struct Db {
    pool: Connection,
    pub files: i64,
    pub dirs: i16,
}

impl Db {
    pub fn new() -> Self {
        let connection = sqlite::open("database.db").unwrap();
        let query = "
	CREATE TABLE IF NOT EXISTS file(file_name VARCHAR(36), file_type VARCHAR(10), path TEXT, content TEXT);
    CREATE TABLE IF NOT EXISTS dir(dir_name VARCHAR(36), path TEXT)
	";

        connection.execute(query).unwrap();

        let files = connection
            .prepare("SELECT count(*) as len FROM file")
            .unwrap();

        let mut file_amount = 0;
        for mut r in files.into_iter().map(|x| x.unwrap()) {
            let len = r.take("len");

            match len {
                Value::Integer(l) => {
                    file_amount = l;
                }
                _ => (),
            };
        }

        Self {
            pool: connection,
            files: file_amount,
            dirs: 0,
        }
    }
    pub fn insert_file(&self, file_name: &str, file_type: &str, path: &str, content: &str) {
        let query = "
	INSERT INTO file VALUES(?, ?, ?, ?) RETURNING *;
	";
        let mut stat = self.pool.prepare(query).unwrap();
        let re = Regex::new(r"^[a-zA-Zа-яА-я]+$").unwrap();
        let f_content = content
            .split(" ")
            .filter(|x| re.is_match(x))
            .into_iter()
            .collect::<Vec<&str>>()
            .join(" ");

        stat.bind((1, file_name)).unwrap();
        stat.bind((2, file_type)).unwrap();
        stat.bind((3, path)).unwrap();
        stat.bind((4, f_content.as_str())).unwrap();
        stat.next().expect("Err during inserting file");
    }
    pub fn get_file_idx(&self, file_idx: i64) -> DbFile {
        let stat = self
            .pool
            .prepare("SELECT *, rowid FROM file WHERE rowid=?;")
            .unwrap()
            .into_iter()
            .bind((1, file_idx))
            .unwrap();

        let mut file_name = String::new();
        let mut file_type = String::new();
        let mut path = String::new();
        let mut content = String::new();
        let mut id = 0;

        for r in stat.into_iter().map(|r| r.unwrap()) {
            file_name.push_str(r.read::<&str, _>("file_name"));
            file_type.push_str(r.read::<&str, _>("file_type"));
            path.push_str(r.read::<&str, _>("path"));
            content.push_str(r.read::<&str, _>("content"));
            id = r.read::<i64, _>("rowid");
        }
        DbFile {
            file_name,
            file_type,
            path,
            content,
            id,
        }
    }
    pub fn get_file(&self, file_name: &str) -> DbFile {
        let stat = self
            .pool
            .prepare("SELECT *, rowid FROM file WHERE file_name=?;")
            .unwrap()
            .into_iter()
            .bind((1, file_name))
            .unwrap();

        let mut file_name = String::new();
        let mut file_type = String::new();
        let mut path = String::new();
        let mut content = String::new();
        let mut id = 0;

        for r in stat.into_iter().map(|r| r.unwrap()) {
            file_name.push_str(r.read::<&str, _>("file_name"));
            file_type.push_str(r.read::<&str, _>("file_type"));
            path.push_str(r.read::<&str, _>("path"));
            content.push_str(r.read::<&str, _>("content"));
            id = r.read::<i64, _>("rowid");
        }
        DbFile {
            file_name,
            file_type,
            path,
            content,
            id,
        }
    }
    pub fn insert_dir(&self, dir_name: &str, path: &str) {
        let query = "
	INSERT INTO dir VALUES(?, ?) RETURNING *;
	";
        let mut stat = self.pool.prepare(query).unwrap();
        stat.bind((1, dir_name)).unwrap();
        stat.bind((2, path)).unwrap();
        stat.next().expect("Err during inserting file");
    }
    pub fn scan(&self, path: &str) -> &Self {
        let (dirs, files) = analyze(path);

        for dir in dirs {
            self.insert_dir(&dir.name, &dir.path);

            self.scan(&dir.path);
        }
        for file in files {
            let content = match file.file_type.as_str() {
                "pdf" => pdf::get(&file.path).unwrap(),
                "docx" => docx::get(&file.path).unwrap(),
                _ => "".to_string(),
            };

            self.insert_file(&file.file_name, &file.file_type, &file.path, &content);

            let db_file = self.get_file(&file.file_name);
            let path = env::current_dir().unwrap();
            let current_path = Path::new(path.as_os_str()).join("dict");
            let re = Regex::new(r"^[a-zA-Zа-яА-я]+$").unwrap();

            // create dict if doesnt exist
            fs::create_dir("dict").ok();
            for (idx, w) in db_file
                .content
                .split(" ")
                .filter(|x| re.is_match(x))
                .enumerate()
            {
                self.insert_word(&db_file.id, idx, w, &current_path.as_path());
            }
        }

        let entries = fs::read_dir("dict")
            .unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()
            .unwrap();

        for path in entries {
            self.sort_file(&path);
        }
        //

        self
    }
    fn write_file_stream(&self, file_path: &PathBuf, append: bool) -> File {
        if append {
            let file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(file_path)
                .unwrap();

            file
        } else {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(file_path)
                .unwrap();

            file
        }
    }
    fn insert_word(&self, file_idx: &i64, idx: usize, word: &str, path: &Path) -> usize {
        let char = &word.chars().take(1).next().unwrap();
        let file_path = path.join(char.to_string().as_str());

        let mut file = self.write_file_stream(&file_path, true);
        let content = format!("{}|{}|{}\n", word, file_idx, idx);
        file.write(content.as_bytes()).unwrap()
    }
    fn sort_file(&self, file_path: &PathBuf) {
        let raw_buff = fs::read(file_path).unwrap();
        let parsed_buff = String::from_utf8(raw_buff).unwrap();
        let content: Vec<&str> = parsed_buff.split("\n").collect();
        let mut file_content = content.clone();
        file_content.sort();
        file_content.sort_by(|a, b| a.len().cmp(&b.len()));

        let mut file = self.write_file_stream(file_path, false);

        for line in file_content {
            let parsed_txt = format!("{}\n", line);
            file.write(parsed_txt.as_bytes())
                .expect("Err during writing the file");
        }
    }
    pub fn search_word(
        &self,
        word: &str,
        lensh_k: f64,
        jer_k: f64,
    ) -> Result<Vec<DictWord>, io::Error> {
        let char = &word.chars().take(1).next().unwrap();
        let path_str = format!("dict/{}", char);
        let path = Path::new(path_str.as_str());

        let raw_file = fs::read(path)?;

        let raw_file_content = String::from_utf8(raw_file).unwrap();
        let file_content: Vec<&str> = raw_file_content.split("\n").collect();

        let mut res: Vec<DictWord> = vec![];

        for line in file_content {
            if line.len() < 1 {
                continue;
            };

            let mut parts = line.split("|");
            let line_word = parts.next().unwrap();
            let file_idx = parts.next().unwrap().parse::<i16>().unwrap();
            let word_idx = parts.next().unwrap().parse::<i32>().unwrap();

            let jer = jaro(word, line_word) * 100.0;
            let lensh = normalized_levenshtein(word, line_word) * 100.0;
            let similarity = ((jer + lensh) / 2.0) as f32;

            let common_average = (lensh + jer) / 2.0;
            let k_average = (lensh_k + jer_k) / 2.0;

            if common_average > k_average {
                let file = self.get_file_idx((file_idx as i64).clone());
                let raw_content = file.content.split(" ").collect::<Vec<&str>>();
                let len_content = raw_content.len();
                let take_idx = if word_idx + 10 > len_content as i32 {
                    len_content
                } else {
                    (word_idx + 10) as usize
                };

                let skip = if word_idx > 11 {
                    word_idx - 10
                } else {
                    word_idx
                } as usize;
                let take = (take_idx) as usize;

                let file_content = format!(
                    "...{}...",
                    raw_content[skip..take].join(" ").replace("\n", " ")
                );

                let resp_word = DictWord {
                    content: word.to_string(),
                    file_idx,
                    word_idx,
                    similarity,
                    file_name: file.file_name,
                    file_path: file.path,
                    file_content: file_content.to_string(),
                };

                res.push(resp_word);
            }
        }

        Ok(res)
    }
}
