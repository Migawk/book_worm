use std::process::Command;

use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Task};

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
    pub search_result: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Scan,
    Search,
    ScanStr(String),
    SearchStr(String),
    SwitchTab(Tab),
    Open(String),
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Scan => {}
            Message::Search => {}
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
            Message::Open(f_name) => {
                println!("{f_name}");
                Command::new("cmd")
                    .args(["/C", "start", "", &f_name])
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
                let mut results = row![].spacing(12);

                for res in &self.search_result {
                    results =
                        results.push(button(res.as_str()).on_press(Message::Open(res.to_string())));
                }

                column![
                    text("Search"),
                    row![
                        text_input("Search by word or phrase", &self.search)
                            .on_input(Message::SearchStr),
                        button("Search").on_press(Message::Search)
                    ]
                    .spacing(12),
                    results
                ]
                .spacing(12)
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
