use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use iced::theme;
use iced::highlighter::{self, Highlighter};
use iced::{Command, executor, Application, Element, Settings, Theme, Length, Font};
use iced::widget::{pick_list, column, horizontal_space, row, container, text, text_editor, button, tooltip};
// use rfd::MessageLevel::Error;


struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
    theme: highlighter::Theme,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    Open,
    Save,
    New,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    FileSaved(Result<PathBuf, Error>),
    ThemeSelected(highlighter::Theme),
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
            theme: highlighter::Theme::SolarizedDark,
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
            Message::ThemeSelected(theme) => {
                self.theme = theme;

                Command::none()
            }
        }
    }


    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            action(new_icon(),"New File", Message::New),
            action(open_icon(),"Open File", Message::Open),
            action(save_icon(),"Save File",  Message::Save),
            horizontal_space(Length::Fill),

            pick_list(highlighter::Theme::ALL,  Some(self.theme), Message::ThemeSelected)
        ].spacing(10);

        let input = text_editor(&self.content)
            .on_edit(Message::Edit)
            .highlight::<Highlighter>(highlighter::Settings {
                theme: self.theme,
                extension: self.path.as_ref().and_then(|path| path.extension()?.to_str()).unwrap_or("rs").to_string(),
            },
                                      |highlighter, _theme| highlighter.to_format());
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
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let content = tokio::fs::read_to_string(&path).await.map(Arc::new).map_err(|err| err.kind()).map_err(Error::IOFailed)?;
    Ok((path, content))
}

fn main() -> iced::Result {
    Editor::run(Settings {
        default_font: Font::MONOSPACE,
        fonts: vec![include_bytes!("../font/iced-icons.ttf").as_slice().into()],
        ..Settings::default()
    })
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


fn new_icon<'a>() -> Element<'a, Message> {
    icon('\u{E800}')
}

fn save_icon<'a>() -> Element<'a, Message> {
    icon('\u{E801}')
}

fn open_icon<'a>() -> Element<'a, Message> {
    icon('\u{F115}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("iced-icons");
    text(codepoint).font(ICON_FONT).into()
}

fn action<'a>(content: Element<'a, Message>, label: &'a str, on_press: Message) -> Element<'a, Message> {
    tooltip(button(container(content).width(20).center_x()).on_press(on_press).padding([5, 5]), label, tooltip::Position::FollowCursor)
        .style(theme::Container::Box)
        .into()
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}

#[derive(Debug, Clone)]
enum Error {
    DialogClose,
    IOFailed(io::ErrorKind),
}

