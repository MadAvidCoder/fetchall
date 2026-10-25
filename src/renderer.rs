use crate::output::{ComponentOutput, OutputRow};
use crate::styled_text::StyledText;
use itertools::{EitherOrBoth, Itertools};

pub fn render(logo: &StyledText, info: &ComponentOutput, logo_gap: usize, label_gap: usize) {
    let logo_width = logo.lines().map(|line| line.width()).max().unwrap_or(40);
    let label_width = info
        .rows
        .iter()
        .filter_map(|row| match row {
            OutputRow::Info { label, .. } => Some(label),
            _ => None,
        })
        .map(|label| label.width())
        .max()
        .unwrap_or(10);

    for paired in logo.lines().zip_longest(info.rows.iter()) {
        let (line, row): (StyledText, &OutputRow) = match paired {
            EitherOrBoth::Both(line, row) => (line, row),
            EitherOrBoth::Left(line) => (line, &OutputRow::Blank),
            EitherOrBoth::Right(row) => (StyledText::new(""), row),
        };
        let logo_pad: usize = if logo_width > line.width() {
            logo_width.saturating_sub(line.width()) + logo_gap
        } else {
            logo_gap
        };
        let row_text: String = match row {
            OutputRow::Info { label, value } => {
                let label_pad: usize = if label_width > label.width() {
                    label_width.saturating_sub(label.width()) + label_gap.saturating_sub(1)
                } else {
                    label_gap.saturating_sub(1)
                };
                format!(
                    "{}{}{} {}",
                    label.render(),
                    " ".repeat(label_pad),
                    if label.text.is_empty() { " " } else { ":" },
                    value.render(),
                )
            }
            OutputRow::Section(section) => section.render(),
            OutputRow::Blank => "".to_string(),
        };

        println!("{}{}{}", line.render(), " ".repeat(logo_pad), row_text,);
    }
    println!();
}
