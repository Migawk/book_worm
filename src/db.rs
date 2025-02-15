use sqlite::Connection;

#[derive(Debug)]
pub struct DbFileWithoutContent {
    pub file_name: String,
    pub file_type: String,
    pub path: String,
}
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
}
impl Db {
    pub fn new() -> Self {
        let connection = sqlite::open("database.db").unwrap();
        let query = "
	CREATE TABLE IF NOT EXISTS file(file_name VARCHAR(36), file_type VARCHAR(10), path TEXT, content TEXT);
    CREATE TABLE IF NOT EXISTS dir(dir_name VARCHAR(36), path TEXT)
	";

        connection.execute(query).unwrap();
        Self { pool: connection }
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
        let query = "
        SELECT * FROM file WHERE file_name=?;
	";
        let mut stat = self.pool.prepare(query).unwrap();
        stat.bind((1, file_name)).unwrap();
        stat.next().expect("Err during obtaining file");

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
    pub fn insert_dir(&self, dir_name: &str, path: &str) {
        let query = "
	INSERT INTO dir VALUES(?, ?) RETURNING *;
	";
        let mut stat = self.pool.prepare(query).unwrap();
        stat.bind((1, dir_name)).unwrap();
        stat.bind((2, path)).unwrap();
        stat.next().expect("Err during inserting file");
    }

    // pub fn search() {
    //     levenshtein(a, b)
    // }
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
}
