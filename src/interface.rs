use std::cmp::Ordering;
use std::process::Command;

use iced::color;
use iced::widget::shader::wgpu::hal::auxil::db::qualcomm;
use iced::widget::{button, column, image, row, scrollable, text, text_input};
use iced::{Element, Task};

use crate::db::{self, DictWord};

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
    pub search_result: Vec<DictWord>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Scan,
    Search,
    ScanStr(String),
    SearchStr(String),
    SwitchTab(Tab),
    Open(DictWord),
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
                self.search_result = vec![];

                for w in self.search.split(" ") {
                    let results = conn.search_word(w, 55.0, 55.0);

                    for res in results {
                        self.search_result.push(res);
                    }
                    self.search_result
                        .sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
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
                let data = db::Db::new();

                column![
                    text("Scan the path to library"),
                    row![
                        text_input("Scan directory", &self.scan).on_input(Message::ScanStr),
                        button("Scan").on_press(Message::Scan),
                    ]
                    .spacing(12),
                    row![
                        text(format!("Files: {}", data.files)),
                        text(format!("Folders: {}", data.dirs)),
                        image("document.svg")
                    ]
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
                .spacing(16);

                for res in self.search_result.iter() {
                    let head = format!("{} - {:.1}%", res.file_name.as_str(), res.similarity);

                    let content = column![
                        row![
                            button("b").on_press(Message::Open(res.clone())),
                            column![
                                text(head.clone()),
                                text(res.file_path.as_str())
                                    .size(12)
                                    .color(color!(0x999999))
                            ]
                        ]
                        .spacing(4),
                        text(res.file_content.clone())
                    ]
                    .spacing(6);
                    results = results.push(content);
                }

                column![scrollable(results)]
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
