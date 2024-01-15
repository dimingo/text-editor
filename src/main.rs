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
    Save,
    New,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    FileSaved(Result<PathBuf, Error>),
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
                self.error = None;
                self.content.edit(action);
                Command::none()
            }
            Message::New => {
                self.path = None;
                self.content = text_editor::Content::new();

                Command::none()
            }
            Message::Open => {
                Command::perform(pick_file(), Message::FileOpened)
            }
            Message::Save => {
                let content = self.content.text();


                Command::perform(save_file(self.path.clone(), content), Message::FileSaved)
            }

            Message::FileSaved(Ok(path)) => {
                self.path = Some(path);
                Command::none()
            }
            Message::FileSaved(Err(error)) => {
                self.error = Some(error);

                Command::none()
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
        let controls = row![
            button("New").on_press(Message::New),
            horizontal_space(4),
            button("Open").on_press(Message::Open),
            horizontal_space(4),
            button("Save").on_press(Message::Save)];

        let input = text_editor(&self.content).on_edit(Message::Edit);
        let status_bar = {
            let status = if let Some(Error::IOFailed(error)) = self.error.as_ref() {
                text(error.to_string())
            } else {
                match self.path.as_deref().and_then(Path::to_str) {
                    Some(path) => text(path).size(14),
                    None => text("New File"),
                }
            }
                ;


            let position = {
                let (line, column) = self.content.cursor_position();

                text(format!("{}:{}", line + 1, column + 1))
            };

            row![status, horizontal_space(Length::Fill), position]
        };

        container(column![controls, input, status_bar].spacing(10)).padding(10).into()
    }
    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let content = tokio::fs::read_to_string(&path).await.map(Arc::new).map_err(|err| err.kind()).map_err(Error::IOFailed)?;
    Ok((path, content))
}

fn main() -> iced::Result {
    Editor::run(Settings::default())
}


async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handle = rfd::AsyncFileDialog::new().set_title("Choose a text file").pick_file().await.ok_or(Error::DialogClose)?;
    load_file(handle.path().to_owned()).await
}

async fn save_file(path: Option<PathBuf>, content: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new().set_title("Choose a file name").save_file().await.ok_or(Error::DialogClose).map(|handle| handle.path().to_owned())?
    };

    tokio::fs::write(&path, &content).await.map_err(|error| Error::IOFailed(error.kind()))?;

    Ok(path)
}


fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}

#[derive(Debug, Clone)]
enum Error {
    DialogClose,
    IOFailed(io::ErrorKind),
}



