use std::process::Command;

use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::{Element, Task};

use crate::db::{self, Db, SearchResult};

#[derive(Debug, Clone)]
pub enum Tab {
    Scanning,
    Searching,
}
impl Default for Tab {
    fn default() -> Self {
        Tab::Scanning
    }
}
#[derive(Default)]
pub struct App {
    pub scan: String,
    pub search: String,
    pub tab: Tab,
    pub search_result: Vec<SearchResult>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Scan,
    Search,
    ScanStr(String),
    SearchStr(String),
    SwitchTab(Tab),
    Open(SearchResult),
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Scan => {
                let conn = db::Db::new();
                conn.scan(&self.scan);
            }
            Message::Search => {
                let conn = db::Db::new();
                let results = conn.search(&self.search);
                self.search_result = vec![];

                for res in results {
                    self.search_result.push(res);
                }
            }
            Message::ScanStr(txt) => {
                self.scan = txt.clone();
            }
            Message::SearchStr(txt) => {
                self.search = txt.clone();
            }
            Message::SwitchTab(tb) => match tb {
                Tab::Scanning => {
                    self.tab = Tab::Scanning;
                }
                Tab::Searching => {
                    self.tab = Tab::Searching;
                }
            },
            Message::Open(res) => {
                Command::new("cmd")
                    .args(["/C", "start", "", &res.file_path])
                    .output()
                    .expect("Err during opening file");
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let tab = match &self.tab {
            Tab::Scanning => {
                column![
                    text("Scan the path to library"),
                    row![
                        text_input("Scan directory", &self.scan).on_input(Message::ScanStr),
                        button("Scan").on_press(Message::Scan)
                    ]
                    .spacing(12)
                ]
            }
            Tab::Searching => {
                let mut results = column![
                    text("Search"),
                    row![
                        text_input("Search by word or phrase", &self.search)
                            .on_input(Message::SearchStr),
                        button("Search").on_press(Message::Search)
                    ]
                    .spacing(12),
                ]
                .spacing(12);

                for (idx, res) in self.search_result.iter().clone().enumerate() {
                    results = results
                        .push(button(res.file_name.as_str()).on_press(Message::Open(res.clone())));
                }

                column![scrollable(results)]
                // results
            }
        };
        let content = column![
            row![
                button("Scanning").on_press(Message::SwitchTab(Tab::Scanning)),
                button("Searching").on_press(Message::SwitchTab(Tab::Searching))
            ],
            row![tab].spacing(12)
        ]
        .spacing(12);

        content.padding(24).into()
    }
}
