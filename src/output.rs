use crate::styled_text::StyledText;

pub enum OutputRow {
    Info {
        label: StyledText,
        value: StyledText,
    },
    Section(StyledText),
    Blank,
}

pub struct ComponentOutput {
    pub rows: Vec<OutputRow>,
}
