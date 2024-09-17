use iced::widget::text_input;
use iced::widget::{column, row};
use iced::{Element, Font, Task};
use protocol::handler::MjProtocolHandler;
use ractor::Actor;
use ractor::ActorRef;
use tracing_subscriber::EnvFilter;
use url::Url;
use webview::MjWebViewMessage;
use webview::MjWebview;

mod cli;
mod dom;
mod protocol;
mod webview;

pub fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_env("MJ_LOG"))
        .init();
    iced::application("MJ", MjBrowser::update, MjBrowser::view)
        .default_font(Font::MONOSPACE)
        .run_with(MjBrowser::new)
}

struct MjBrowser {
    view: Option<ActorRef<MjWebViewMessage>>,
    url_or_query: String,
}

#[derive(Debug, Clone)]
enum Message {
    Initializing,
    UrlBarChanged(String),
    Navigate,
    Navigated(ActorRef<MjWebViewMessage>),
}

impl MjBrowser {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                view: None,
                url_or_query: "file:///home/matt/code/mj/resources/views/new.html".to_string(),
            },
            Task::perform(async {}, |_| Message::Initializing),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Initializing => Task::perform(
                async {
                    Actor::spawn(
                        Some("mj:protocol_handler".to_string()),
                        MjProtocolHandler,
                        (),
                    )
                    .await
                    .unwrap()
                },
                |_| Message::Navigate,
            ),
            Message::UrlBarChanged(value) => {
                self.url_or_query = value;
                Task::none()
            }
            Message::Navigate => {
                let url = Url::parse(&self.url_or_query).unwrap();

                Task::perform(
                    async { Actor::spawn(None, MjWebview, (url,)).await.unwrap() },
                    |(actor, _)| Message::Navigated(actor),
                )
            }
            Message::Navigated(actor) => {
                if let Some(old) = self.view.replace(actor) {
                    old.stop(None)
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let tab_bar = row![text_input(
            "Search with DuckDuckGo or enter address",
            &self.url_or_query
        )
        .id(text_input::Id::unique())
        .on_input(Message::UrlBarChanged)
        .on_submit(Message::Navigate)];
        column![tab_bar].into()
    }
}
