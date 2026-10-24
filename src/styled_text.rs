use owo_colors::{OwoColorize, Style};
use unicode_width::UnicodeWidthStr;

macro_rules! style_methods {
    ($($name:ident),* $(,)?) => {
        $(
            #[allow(dead_code)]
            pub fn $name(mut self) -> Self {
                self.style = self.style.$name();
                self
            }
        )*
    };
}

pub struct StyledText {
    pub text: String,
    style: Style,
}

impl StyledText {
    pub fn new(text: impl Into<String>) -> Self {
        StyledText {
            text: text.into(),
            style: Style::new(),
        }
    }

    style_methods!(
        black,
        red,
        green,
        yellow,
        blue,
        magenta,
        cyan,
        white,
        bright_black,
        bright_red,
        bright_green,
        bright_yellow,
        bright_blue,
        bright_magenta,
        bright_cyan,
        bright_white,
        on_black,
        on_red,
        on_green,
        on_yellow,
        on_blue,
        on_magenta,
        on_cyan,
        on_white,
        bold,
        dimmed,
        italic,
        underline,
        strikethrough,
        reversed,
    );

    pub fn lines(&self) -> impl Iterator<Item = Self> + '_ {
        self.text.lines().map(|line| Self {
            text: line.to_string(),
            style: self.style,
        })
    }

    pub fn width(&self) -> usize {
        UnicodeWidthStr::width(&self.text as &str)
    }

    pub fn render(&self) -> String {
        self.text.style(self.style).to_string()
    }
}

impl From<&str> for StyledText {
    fn from(text: &str) -> Self {
        StyledText::new(text)
    }
}
