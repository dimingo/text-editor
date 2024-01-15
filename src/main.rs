use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use iced::{Command, executor, Application, Element, Settings, Theme, Length};
use iced::widget::{column, horizontal_space, row, container, text, text_editor, button};
// use rfd::MessageLevel::Error;


struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    Open,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
}

impl Application for Editor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();


    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (Self {
            path: None,
            content: text_editor::Content::new(),
            error: None,
        }, Command::perform(
            load_file(default_file()),
            Message::FileOpened)
        )
    }

    fn title(&self) -> String {
        String::from("Dimingo's Editor")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
                Command::none()
            }
            Message::Open => {
                Command::perform(pick_file(), Message::FileOpened)
            }
            Message::FileOpened(Ok((path, content))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with(&content);
                Command::none()
            }

            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![button("Open").on_press(Message::Open)];
        let input = text_editor(&self.content).on_edit(Message::Edit);
        let file_path = match self.path.as_deref().and_then(Path::to_str) {
            Some(path) => text(path).size(14),
            None => text(""),
        };

        let position = {
            let (line, column) = self.content.cursor_position();

            text(format!("{}:{}", line + 1, column + 1))
        };


        let status_bar = row![file_path, horizontal_space(Length::Fill), position];

        container(column![controls, input, status_bar].spacing(10)).padding(10).into()
    }
    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let content = tokio::fs::read_to_string(&path).await.map(Arc::new).map_err(|err| err.kind()).map_err(Error::IO)?;
    Ok((path, content))
}

fn main() -> iced::Result {
    Editor::run(Settings::default())
}


async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handle = rfd::AsyncFileDialog::new().set_title("Choose a text file").pick_file().await.ok_or(Error::DialogClose)?;
    load_file(handle.path().to_owned()).await
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}

#[derive(Debug, Clone)]
enum Error {
    DialogClose,
    IO(io::ErrorKind),
}




