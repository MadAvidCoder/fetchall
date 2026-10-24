use crate::component::Component;
use crate::{
    output::{ComponentOutput, OutputRow},
    styled_text::StyledText,
};
use anyhow::Result;
use hostname;
use std::env;

pub struct UserComponent;

impl Component for UserComponent {
    fn title(&self) -> &str {
        "User Info"
    }

    fn collect(&self) -> Result<ComponentOutput> {
        let username = env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or("Unknown".to_string());
        let hostname = hostname::get()?
            .into_string()
            .unwrap_or("Unknown".to_string());
        Ok(ComponentOutput {
            rows: vec![OutputRow::Section(
                StyledText::new(format!("{username}@{hostname}"))
                    .bold()
                    .italic()
                    .bright_red(),
            )],
        })
    }
}
