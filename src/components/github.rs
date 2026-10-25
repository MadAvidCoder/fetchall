use crate::component::Component;
use crate::config::GithubConfig;
use crate::output::{ComponentOutput, OutputRow};
use crate::styled_text::StyledText;
use anyhow::Result;
use git2::Repository;

pub struct GithubComponent<'a> {
    config: &'a GithubConfig,
    colour: &'a str,
    show_section_titles: &'a bool,
}

impl<'a> GithubComponent<'a> {
    pub fn new(config: &'a GithubConfig, colour: &'a str, show_section_titles: &'a bool) -> Self {
        Self {
            config,
            colour,
            show_section_titles,
        }
    }
}

impl Component for GithubComponent<'_> {
    fn title(&self) -> &str {
        "GitHub"
    }
    fn collect(&self) -> Result<ComponentOutput> {
        let mut output = ComponentOutput { rows: Vec::new() };
        if !self.config.enabled {
            return Ok(output);
        }

        if *self.show_section_titles {
            if self.colour != "auto" {
                output.rows.push(OutputRow::Section(
                    StyledText::new(self.title().to_string())
                        .apply_style(self.colour)
                        .bold()
                        .underline(),
                ));
            } else {
                output.rows.push(OutputRow::Section(
                    StyledText::new(self.title().to_string()).bold().underline(),
                ));
            }
        }

        let repo = Repository::discover(".")?;
        let remote = repo.find_remote("origin")?;
        let url = remote.url().unwrap_or("Unknown URL");

        if self.colour != "auto" {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("url".to_string())
                    .apply_style(self.colour)
                    .bold(),
                value: StyledText::new(url.to_string()),
            });
        } else {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("url".to_string()).yellow().bold(),
                value: StyledText::new(url.to_string()),
            });
        }

        Ok(output)
    }
}
