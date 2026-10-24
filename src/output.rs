use std::ops::{Add, AddAssign};

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

impl Add for ComponentOutput {
    type Output = Self;
    fn add(mut self, other: Self) -> Self {
        self.rows.extend(other.rows);
        self
    }
}

impl AddAssign for ComponentOutput {
    fn add_assign(&mut self, other: Self) {
        self.rows.extend(other.rows);
    }
}

impl FromIterator<ComponentOutput> for ComponentOutput {
    fn from_iter<T: IntoIterator<Item = ComponentOutput>>(iter: T) -> Self {
        let mut output = Self { rows: Vec::new() };

        for component in iter {
            output.rows.extend(component.rows);
        }

        output
    }
}
