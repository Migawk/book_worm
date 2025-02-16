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
}
pub struct Db {
    pool: Connection,
    files: i64,
    dirs: i16,
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
        stat.bind((1, file_name)).unwrap();
        stat.bind((2, file_type)).unwrap();
        stat.bind((3, path)).unwrap();
        stat.bind((4, content)).unwrap();
        stat.next().expect("Err during inserting file");

        // let mut file_name = String::new();
        // let mut file_type = String::new();
        // let mut path = String::new();
        // let mut content = String::new();

        // for r in stat.into_iter().map(|r| r.unwrap()) {
        //     file_name.push_str(r.read::<&str, _>("file_name"));
        //     file_type.push_str(r.read::<&str, _>("file_type"));
        //     path.push_str(r.read::<&str, _>("path"));
        //     content.push_str(r.read::<&str, _>("content"));
        // }
        // DbFile {
        //     file_name,
        //     file_type,
        //     path,
        //     content,
        // }
    }
    pub fn get_file(&self, file_name: &str) -> DbFile {
        let stat = self
            .pool
            .prepare("SELECT * FROM file WHERE file_name=?;")
            .unwrap()
            .into_iter()
            .bind((1, file_name))
            .unwrap();

        let mut file_name = String::new();
        let mut file_type = String::new();
        let mut path = String::new();
        let mut content = String::new();

        for r in stat.into_iter().map(|r| r.unwrap()) {
            file_name.push_str(r.read::<&str, _>("file_name"));
            file_type.push_str(r.read::<&str, _>("file_type"));
            path.push_str(r.read::<&str, _>("path"));
            content.push_str(r.read::<&str, _>("content"));
        }
        DbFile {
            file_name,
            file_type,
            path,
            content,
        }
    }
    pub fn get_files(&self, limit: i64, offset: i64) -> Vec<String> {
        let stat = self
            .pool
            .prepare("SELECT * FROM file LIMIT :lim OFFSET :offs;")
            .unwrap()
            .into_iter()
            .bind((":lim", limit))
            .unwrap()
            .bind((":offs", offset))
            .unwrap();

        let mut names: Vec<String> = vec![];

        for r in stat.into_iter().map(|r| r.unwrap()) {
            let dat = r.read::<&str, _>("file_name");
            names.push(dat.to_string());
        }

        names
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
    pub fn get_dir(&self, file_name: &str) -> Dir {
        let query = "
        SELECT * FROM dir WHERE dir_name=?;
	";
        let mut stat = self.pool.prepare(query).unwrap();
        stat.bind((1, file_name)).unwrap();
        stat.next().expect("Err during obtaining file");

        let mut dir_name = String::new();
        let mut path = String::new();
        for r in stat.into_iter().map(|r| r.unwrap()) {
            dir_name.push_str(r.read::<&str, _>("file_name"));
            path.push_str(r.read::<&str, _>("path"));
        }
        Dir {
            name: dir_name,
            path,
        }
    }
    pub fn create_virtual_db(&self) -> &Self {
        self.pool
            .execute(
                "
        CREATE VIRTUAL TABLE IF NOT EXISTS file_v USING fts5(file_name, file_type, path, content);
        ",
            )
            .expect("Err during creating virtual table 'file'");

        self.pool
            .execute(
                "
        INSERT INTO file_v(file_name, file_type, path, content)
        SELECT file_name, file_type, path, content FROM file
        ",
            )
            .unwrap();

        self
    }
    pub fn search(&self, searching: &str) -> Vec<(String, String, usize, f64, f64)> {
        let files = self.get_files(self.files, 0);
        let mut resp: Vec<(String, String, usize, f64, f64)> = vec![];

        for f_name in files {
            let file = self.get_file(&f_name.as_str());
            let words = file.content.split(" ").enumerate();

            for (idx, w) in words {
                let word = w.trim().lines().collect::<Vec<&str>>().join(" ");

                let jer = jaro(word.as_str(), searching) * 100.0;
                let lensh = normalized_levenshtein(word.as_str(), searching) * 100.0;

                if lensh > 60.0 && jer > 60.0 {
                    resp.push((f_name.clone(), word, idx, lensh, jer));
                }
            }
        }

        resp.sort_by(|a, b| {
            ((100.0 - a.3).abs() + (100.0 - a.4).abs())
                .total_cmp(&((100.0 - b.3).abs() + (100.0 - b.4).abs()))
        });
        resp
    }
    pub fn scan(&self, path: &str) -> &Self {
        let (dirs, files) = analyze(path);

        for dir in dirs {
            self.insert_dir(&dir.name, &dir.path);

            println!("adding DIR :  {}", dir.name);
            self.scan(&dir.path);
        }
        for file in files {
            let content = match file.file_type.as_str() {
                "pdf" => pdf::get(&file.path).unwrap(),
                "docx" => docx::get(&file.path).unwrap(),
                _ => "".to_string(),
            };

            println!("adding FILE: {}", file.file_name);
            self.insert_file(&file.file_name, &file.file_type, &file.path, &content);
        }

        self
    }
}
