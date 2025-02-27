mod crawler;
mod db;
mod docx;
mod interface;
mod pdf;

use iced::{self, Task};
use interface::{App, Tab};

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().collect();

    let init = App {
        scan: String::from(args[0].clone()),
        search: String::new(),
        tab: Tab::Scanning,
        search_result: vec![],
    };

    iced::application("Book Worm", App::update, App::view).run_with(|| (init, Task::none()))
}
