use crate::component::Component;
use crate::config::HackatimeConfig;
use crate::output::ComponentOutput;
use crate::output::OutputRow;
use crate::styled_text::StyledText;

pub struct HackatimeComponent {
    config: &'a HackatimeConfig,
    colour: &'a str,
    show_section_titles: &'a bool,
}

impl<'a> HackatimeComponent<'a> {
    pub fn new(
        config: &'a HackatimeConfig,
        colour: &'a str,
        show_section_titles: &'a bool,
    ) -> Self {
        Self {
            config,
            colour,
            show_section_titles,
        }
    }
}

impl Component for HackatimeComponent<'_> {
    fn title(&self) -> &str {
        "Hackatime"
    }
    fn collect(&self) -> anyhow::Result<ComponentOutput> {
        let mut output = ComponentOutput { rows: Vec::new() };
        if !self.config.enabled {
            return Ok(output);
        }
        if *self.show_section_titles {
            if self.colour != "auto" {
                output.rows.push(OutputRow::Section(
                    self.title()
                        .into()
                        .bold()
                        .apply_style(self.colour)
                        .underline(),
                ))
            } else {
                output
                    .rows
                    .push(OutputRow::Section(self.title().into().bold().underline()))
            }
        }

        Ok(output)
    }
}
