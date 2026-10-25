use crate::component::Component;
use crate::config::DevEnvConfig;
use crate::helpers::execute_command;
use crate::output::{ComponentOutput, OutputRow};
use crate::styled_text::StyledText;
use anyhow::Result;

pub struct DevEnvComponent<'a> {
    config: &'a DevEnvConfig,
}

impl<'a> DevEnvComponent<'a> {
    pub fn new(config: &'a DevEnvConfig) -> Self {
        Self { config }
    }
}

impl Component for DevEnvComponent<'_> {
    fn title(&self) -> &str {
        "Development Environment"
    }

    fn collect(&self) -> Result<ComponentOutput> {
        let mut output = ComponentOutput { rows: Vec::new() };
        if !self.config.enabled {
            return Ok(output);
        }

        output.rows.push(OutputRow::Section(
            StyledText::new(self.title()).bold().underline(),
        ));

        if self.config.show_project || self.config.show_path || self.config.show_project_tools {
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

            if self.config.show_project {
                let projects = if detected.is_empty() {
                    return Err(anyhow::anyhow!("No recognized project files found"));
                } else {
                    detected.join(", ")
                };

                output.rows.push(OutputRow::Info {
                    label: StyledText::new("project").cyan().bold(),
                    value: StyledText::new(projects),
                });
            }

            if self.config.show_path {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("path").blue().bold(),
                    value: StyledText::new(project_root.display().to_string()),
                });
            }

            if self.config.show_project_tools {
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
            }
        }

        if self.config.show_system_tools {
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
        }

        output.rows.push(OutputRow::Blank);

        Ok(output)
    }
}
