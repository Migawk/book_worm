mod db;
use db::Db;

mod crawler;
mod docx;
mod pdf;

fn main() {
    // DEFAULT FLOW
    let args: Vec<String> = std::env::args().collect();
    let db = Db::new();

    if args.len() > 1 {
        let action = args[1].clone();

        match action.as_str() {
            "scan" => {
                let path = args[2].clone();
                db.scan(path.as_str());
                db.create_virtual_db();
                println!("Scanning is ended. You may proceed with search action.");
            }
            "search" => {
                let searching = args[2].clone();
                let resp = db.search(searching.as_str());
                for r in resp {
                    println!("{}|{}|{}|{}|{}|", r.0, r.1, r.2, r.3, r.4);
                }
            },
            _ => {
                println!("Failed action");
            }
        }
    } else {
        println!(
            "Usage:
        book_worm <action> [param1 param2 ...]
        "
        );
    }

    // SEARCHING BY PHRASE
    // let resp = db.search("ведомости");
    // dbg!(resp);
}
