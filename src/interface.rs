use std::process::Command;

use iced::color;
use iced::widget::{button, checkbox, column, image, row, scrollable, slider, text, text_input};
use iced::{Element, Task};
use rfd::FileDialog;

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
    pub ai: bool,
    pub similarity: f32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Scan,
    Search,
    SearchStr(String),
    SwitchTab(Tab),
    Open(DictWord),
    SwitchAI(bool),
    Slide(f32),
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Scan => {
                let conn = db::Db::new();

                let path = FileDialog::new().pick_folder().unwrap();
                self.scan = String::from(path.to_str().unwrap());

                conn.scan(&self.scan);
            }
            Message::Search => {
                let conn = db::Db::new();
                self.search_result = vec![];

                for w in self.search.split(" ") {
                    let results = conn.search_word(
                        w,
                        self.similarity.into(),
                        self.similarity.into(),
                        self.ai,
                    );

                    match results {
                        Ok(results_ok) => {
                            for res in results_ok {
                                self.search_result.push(res);
                            }
                            self.search_result
                                .sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
                        }
                        Err(_) => {}
                    }
                }
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
            Message::SwitchAI(v) => self.ai = v,
            Message::Slide(v) => {
                self.similarity = v;
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let tab = match &self.tab {
            Tab::Scanning => {
                let data = db::Db::new();

                let scanning_path = format!("Scanning path path is: {}", self.scan);

                column![
                    text("Scan the path to library"),
                    row![text(scanning_path), button("Scan").on_press(Message::Scan),].spacing(12),
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

                let current_similarity = format!("{:.0}%", self.similarity);
                column![
                    row![
                        checkbox("AI", self.ai).on_toggle(Message::SwitchAI),
                        row![
                            text(current_similarity),
                            slider(55.0..=100.0, self.similarity, Message::Slide)
                        ]
                        .spacing(2)
                    ]
                    .spacing(12),
                    scrollable(results)
                ]
            }
        };
        let content = column![
            row![
                button("Scanning").on_press(Message::SwitchTab(Tab::Scanning)),
                button("Searching").on_press(Message::SwitchTab(Tab::Searching))
            ],
            row![tab].spacing(12),
        ]
        .spacing(12);

        content.padding(24).into()
    }
}
