use crate::component::Component;
use crate::config::UserConfig;
use crate::{
    output::{ComponentOutput, OutputRow},
    styled_text::StyledText,
};
use anyhow::Result;
use hostname;
use std::env;

pub struct UserComponent<'a> {
    config: &'a UserConfig,
    colour: &'a str,
    show_section_titles: &'a bool,
}

impl<'a> UserComponent<'a> {
    pub fn new(config: &'a UserConfig, colour: &'a str, show_section_titles: &'a bool) -> Self {
        Self {
            config,
            colour,
            show_section_titles,
        }
    }
}

impl Component for UserComponent<'_> {
    fn title(&self) -> &str {
        "User Info"
    }

    fn collect(&self) -> Result<ComponentOutput> {
        if !self.config.enabled {
            return Ok(ComponentOutput { rows: Vec::new() });
        }
        let username = if self.config.show_user {
            env::var("USER").or_else(|_| env::var("USERNAME")).ok()
        } else {
            None
        };
        let hostname = if self.config.show_hostname {
            hostname::get()?.into_string().ok()
        } else {
            None
        };
        Ok(ComponentOutput {
            rows: vec![OutputRow::Section(
                if let (Some(username), Some(hostname)) = (username.as_deref(), hostname.as_deref())
                {
                    StyledText::new(format!("{username}@{hostname}"))
                        .bold()
                        .italic()
                        .bright_red()
                } else if let Some(username) = username {
                    if self.colour != "auto" {
                        StyledText::new(username)
                            .apply_style(self.colour)
                            .bold()
                            .italic()
                    } else {
                        StyledText::new(username).bold().italic().bright_red()
                    }
                } else if let Some(hostname) = hostname {
                    if self.colour != "auto" {
                        StyledText::new(hostname)
                            .apply_style(self.colour)
                            .bold()
                            .italic()
                    } else {
                        StyledText::new(hostname).bold().italic().bright_red()
                    }
                } else {
                    if self.colour != "auto" {
                        StyledText::new("Unknown")
                            .apply_style(self.colour)
                            .bold()
                            .italic()
                    } else {
                        StyledText::new("Unknown").bold().italic().bright_red()
                    }
                },
            )],
        })
    }
}
