use std::process::Command;

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

        let project_root = current_dir
            .ancestors()
            .find_map(|dir| {
                info_files
                    .iter()
                    .any(|(file, _)| dir.join(file).is_file())
                    .then(|| dir.to_path_buf())
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No recognized project files found in the current directory or its ancestors"
                )
            })?;

        let detected: Vec<&str> = info_files
            .iter()
            .filter(|(file, _)| project_root.join(file).is_file())
            .map(|(_, lang)| *lang)
            .collect();

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
            value: StyledText::new(project_root.display().to_string()),
        });

        for lang in detected {
            for (label, cmd, args) in match lang {
                "Rust" => vec![
                    ("rustc", "rustc", &["--version"]),
                    ("cargo", "cargo", &["--version"]),
                ],
                "Node.js" => vec![
                    ("node", "node", &["--version"]),
                    ("npm", "npm.cmd", &["--version"]),
                ],
                "Python" => vec![
                    ("python", "python", &["--version"]),
                    ("pip", "pip", &["--version"]),
                ],
                "Go" => vec![
                    ("go", "go", &["version"]),
                    ("gofmt", "gofmt", &["--version"]),
                ],
                "Java / Maven" => vec![
                    ("mvn", "mvn", &["--version"]),
                    ("java", "java", &["-version"]),
                ],
                "Java / Gradle" => vec![
                    ("gradle", "gradle", &["--version"]),
                    ("java", "java", &["-version"]),
                ],
                "C / C++" => vec![
                    ("cmake", "cmake", &["--version"]),
                    ("gcc", "gcc", &["--version"]),
                    ("g++", "g++", &["--version"]),
                ],
                _ => unreachable!(),
            } {
                if let Some(version) = execute_command(cmd, args) {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new(label).green().bold(),
                        value: StyledText::new(version),
                    });
                }
            }
        }

        for (label, cmd, args) in [
            ("git", "git", &["--version"]),
            ("docker", "docker", &["--version"]),
        ] {
            if let Some(version) = execute_command(cmd, args) {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new(label).green().bold(),
                    value: StyledText::new(version),
                });
            }
        }

        output.rows.push(OutputRow::Blank);

        Ok(output)
    }
}

fn execute_command(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let text = if !stdout.trim().is_empty() {
        stdout.trim()
    } else {
        stderr.trim()
    };

    Some(text.lines().next()?.trim().to_string())
}
