use anyhow::Result;

use crate::component::Component;
use crate::output::{ComponentOutput, OutputRow};
use crate::styled_text::StyledText;

pub struct DevEnvironmentComponent;

impl Component for DevEnvironmentComponent {
    fn title(&self) -> &str {
        "Development Environment"
    }

    fn collect(&self) -> Result<ComponentOutput> {
        let mut output = ComponentOutput { rows: Vec::new() };
        output.rows.push(OutputRow::Section(
            StyledText::new(self.title()).bold().underline(),
        ));

        let current_dir = std::env::current_dir()?;

        let info_files = [
            ("Cargo.toml", "Rust"),
            ("package.json", "Node.js"),
            ("pyproject.toml", "Python"),
            ("requirements.txt", "Python"),
            ("go.mod", "Go"),
            ("pom.xml", "Java / Maven"),
            ("build.gradle", "Java / Gradle"),
            ("CMakeLists.txt", "C / C++"),
        ];

        let detected: Vec<&str> = info_files.iter().filter(|(file, _)| current_dir.join(file).is_file()).map(|(_, lang)| *lang).collect();

        let projects = if detected.is_empty() {
            return Err(anyhow::anyhow!("No recognized project files found"));
        } else {
            detected.join(", ")
        };
        output.rows.push(OutputRow::Info {
            label: StyledText::new("project").cyan().bold(),
            value: StyledText::new(projects),
        });
        output.rows.push(OutputRow::Info {
            label: StyledText::new("path").blue().bold(),
            value: StyledText::new(current_dir.display().to_string()),
        });

        output.rows.push(OutputRow::Blank);

        Ok(output)
    }
}
