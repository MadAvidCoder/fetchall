use anyhow::Context;

use crate::component::Component;
use crate::config::HackatimeConfig;
use crate::hackatime_auth::{get_client, make_request_with_token};
use crate::output::ComponentOutput;
use crate::output::OutputRow;
use crate::styled_text::StyledText;

pub struct HackatimeComponent<'a> {
    config: &'a HackatimeConfig,
    colour: &'a str,
    show_section_titles: &'a bool,
    show_section_gap: &'a bool,
}

impl<'a> HackatimeComponent<'a> {
    pub fn new(
        config: &'a HackatimeConfig,
        colour: &'a str,
        show_section_titles: &'a bool,
        show_section_gap: &'a bool,
    ) -> Self {
        Self {
            config,
            colour,
            show_section_titles,
            show_section_gap,
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
                    StyledText::new(self.title())
                        .bold()
                        .apply_style(self.colour)
                        .underline(),
                ))
            } else {
                output.rows.push(OutputRow::Section(
                    StyledText::new(self.title()).bold().underline(),
                ))
            }
        }

        if self.config.show_hours {
            let today = make_request_with_token(get_client()?.get(format!(
                "https://hackatime.hackclub.com/api/v1/authenticated/hours?start_date={}&end_date={}",
                chrono::Local::now().format("%Y-%m-%d"),
                chrono::Local::now().format("%Y-%m-%d"),
            )))?
            .json::<serde_json::Value>()?
            .get("total_seconds")
            .and_then(|v| {
                format!(
                    "{}h {}m",
                    v.as_u64().unwrap_or(0) / 3600,
                    (v.as_u64().unwrap_or(0) % 3600) / 60
                )
                .into()
            })
            .unwrap_or("0h 0m".into());
            let week = make_request_with_token(get_client()?.get(format!(
                "https://hackatime.hackclub.com/api/v1/authenticated/hours?start_date={}&end_date={}",
                (chrono::Local::now() - chrono::Duration::days(7)).format("%Y-%m-%d"),
                chrono::Local::now().format("%Y-%m-%d"),
            )))?
            .json::<serde_json::Value>()?
            .get("total_seconds")
            .and_then(|v| {
                format!(
                    "{}h {}m",
                    v.as_u64().unwrap_or(0) / 3600,
                    (v.as_u64().unwrap_or(0) % 3600) / 60
                )
                .into()
            })
            .unwrap_or("0h 0m".into());
            let all_time = make_request_with_token(get_client()?.get(format!(
                "https://hackatime.hackclub.com/api/v1/authenticated/hours?start_date=1970-01-01&end_date={}",
                chrono::Local::now().format("%Y-%m-%d"),
            )))?
            .json::<serde_json::Value>()?
            .get("total_seconds")
            .and_then(|v| {
                format!(
                    "{}h {}m",
                    v.as_u64().unwrap_or(0) / 3600,
                    (v.as_u64().unwrap_or(0) % 3600) / 60
                )
                .into()
            })
            .unwrap_or("0h 0m".into());
            if self.colour != "auto" {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("hours").apply_style(self.colour),
                    value: StyledText::new(format!(
                        "{} today | {} week | {} total",
                        today, week, all_time
                    ))
                    .apply_style(self.colour),
                })
            } else {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("hours").green(),
                    value: StyledText::new(format!(
                        "{} today | {} week | {} total",
                        today, week, all_time
                    )),
                })
            }
        }

        if self.config.show_streak || self.config.show_languages {
            let stats = make_request_with_token(get_client()?.get(
                "https://hackatime.hackclub.com/api/v1/users/my/stats?start_date=1970-01-01",
            ))?
            .json::<serde_json::Value>()?;

            if self.config.show_streak {
                let streak = stats
                    .get("data")
                    .context("Couldn't get stats data")?
                    .get("streak")
                    .context("Couldn't get streak")?
                    .as_u64()
                    .unwrap_or(0);

                let streak_hrs = make_request_with_token(get_client()?.get(format!(
                    "https://hackatime.hackclub.com/api/v1/authenticated/hours?start_date={}&end_date={}",
                    (chrono::Local::now() - chrono::Duration::days(streak as i64)).format("%Y-%m-%d"),
                    chrono::Local::now().format("%Y-%m-%d"),
                )))?
                .json::<serde_json::Value>()?
                .get("total_seconds")
                .and_then(|v| {
                    format!(
                        "{}h {}m",
                        v.as_u64().unwrap_or(0) / 3600,
                        (v.as_u64().unwrap_or(0) % 3600) / 60
                    )
                    .into()
                })
                .unwrap_or("0h 0m".into());
                if self.colour != "auto" {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("streak").apply_style(self.colour),
                        value: StyledText::new(format!("{} days | {}", streak, streak_hrs))
                            .apply_style(self.colour),
                    })
                } else {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("streak").green(),
                        value: StyledText::new(format!("{} days | {}", streak, streak_hrs)),
                    })
                }
            }

            if self.config.show_languages {
                let top_languages = stats
                    .get("data")
                    .context("Couldn't get stats data")?
                    .get("languages")
                    .context("Couldn't get top languages")?
                    .as_array()
                    .unwrap_or(&vec![])[..3]
                    .iter()
                    .map(|lang| {
                        format!(
                            "{} ({}%)",
                            lang.get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown"),
                            lang.get("percent").and_then(|v| v.as_f64()).unwrap_or(0f64)
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(" | ");
                if self.colour != "auto" {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("languages").apply_style(self.colour),
                        value: StyledText::new(top_languages),
                    })
                } else {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("languages").green(),
                        value: StyledText::new(top_languages),
                    })
                }
            }
        }

        if self.config.show_project {
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
                ("project.godot", "Godot"),
            ];

            if let Some(project_root) = current_dir
                .ancestors()
                .find_map(|dir| {
                    info_files
                        .iter()
                        .any(|(file, _)| dir.join(file).is_file())
                        .then(|| dir.to_path_buf())
                })
                .and_then(|dir| {
                    dir.file_name()
                        .and_then(|name| name.to_str())
                        .map(|s| s.to_string())
                })
            {
                let binding = make_request_with_token(get_client()?.get(format!(
                    "https://hackatime.hackclub.com/api/v1/users/my/stats?filter_by_project={project_root}"
                )))
                .context("Request failed")?
                .json::<serde_json::Value>()?;
                let project_info = binding.get("data").unwrap_or_default();
                let project_time = project_info
                    .get("total_seconds")
                    .and_then(|v| {
                        format!(
                            "{}h {}m",
                            v.as_u64().unwrap_or(0) / 3600,
                            (v.as_u64().unwrap_or(0) % 3600) / 60
                        )
                        .into()
                    })
                    .unwrap_or("0h 0m".into());
                let binding = Vec::new();
                let project_language = project_info
                    .get("languages")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&binding)
                    .iter()
                    .next()
                    .map(|lang| {
                        format!(
                            "{}% {}",
                            lang.get("percent").and_then(|n| n.as_f64()).unwrap_or(0.0),
                            lang.get("name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("Unknown")
                        )
                    })
                    .unwrap_or("Unknown".into());
                if self.colour != "auto" {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("project").apply_style(self.colour),
                        value: StyledText::new(format!(
                            "{} | {} | {} total",
                            project_root, project_language, project_time
                        ))
                        .apply_style(self.colour),
                    })
                } else {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("project").green(),
                        value: StyledText::new(format!(
                            "{} | {} | {} total",
                            project_root, project_language, project_time
                        )),
                    })
                }
            }
        }

        if *self.show_section_gap {
            output.rows.push(OutputRow::Blank);
        }

        Ok(output)
    }
}
