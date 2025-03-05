mod crawler;
mod db;
mod docx;
mod interface;
mod pdf;
mod ai;

use std::env;

use iced::{self, Task};
use interface::{App, Tab};

fn main() -> iced::Result {
    let scan_path_init = env::current_dir().unwrap();
    
    let init = App {
        scan: String::from(scan_path_init.to_string_lossy()),
        search: String::new(),
        tab: Tab::Scanning,
        search_result: vec![],
        ai: false,
        similarity: 55.0
    };

    iced::application("Book Worm", App::update, App::view).run_with(|| (init, Task::none()))
}
