use crate::component::Component;
use crate::components::github_repo::fetch_all_pages;
use crate::config::GithubProfileConfig;
use crate::github_auth;
use crate::output::{ComponentOutput, OutputRow};
use crate::styled_text::StyledText;
use anyhow::{Context, Result};
use chrono::{Duration, Utc};

pub struct GithubProfileComponent<'a> {
    config: &'a GithubProfileConfig,
    colour: &'a str,
    show_section_titles: &'a bool,
    show_section_gap: &'a bool,
}

impl<'a> GithubProfileComponent<'a> {
    pub fn new(
        config: &'a GithubProfileConfig,
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

impl Component for GithubProfileComponent<'_> {
    fn title(&self) -> &str {
        "GitHub Profile Stats"
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

        if self.config.show_username || self.config.show_repos || self.config.show_followers {
            let profile = github_auth::make_request_with_token(
                github_auth::get_client()?.get("https://api.github.com/user"),
            )
            .context("Failed to fetch GitHub profile information")?
            .json::<serde_json::Value>()
            .context("Failed to parse GitHub profile information")?;

            if self.config.show_username {
                let username = profile
                    .get("login")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");

                let name = profile
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");

                if self.colour != "auto" {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("username".to_string())
                            .apply_style(self.colour)
                            .bold(),
                        value: StyledText::new(format!("@{} | {}", username, name)),
                    });
                } else {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("username".to_string())
                            .bright_yellow()
                            .bold(),
                        value: StyledText::new(format!("@{} | {}", username, name)),
                    });
                }
            }

            if self.config.show_repos {
                let public_repos = profile
                    .get("public_repos")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let private_repos = fetch_all_pages(
                    "https://api.github.com/user/repos",
                    &[
                        ("visibility", "private"),
                        ("affiliation", "owner,collaborator"),
                    ],
                    None,
                )?
                .len();

                if self.colour != "auto" {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("repositories".to_string())
                            .apply_style(self.colour)
                            .bold(),
                        value: StyledText::new(format!(
                            "{} public | {} private",
                            public_repos, private_repos
                        )),
                    });
                } else {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("repositories".to_string())
                            .bright_yellow()
                            .bold(),
                        value: StyledText::new(format!(
                            "{} public | {} private",
                            public_repos, private_repos
                        )),
                    });
                }
            }

            if self.config.show_followers {
                let followers = profile
                    .get("followers")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let following = profile
                    .get("following")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if self.colour != "auto" {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("followers".to_string())
                            .apply_style(self.colour)
                            .bold(),
                        value: StyledText::new(format!(
                            "{} followers | {} following",
                            followers, following
                        )),
                    });
                } else {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("followers".to_string())
                            .bright_yellow()
                            .bold(),
                        value: StyledText::new(format!(
                            "{} followers | {} following",
                            followers, following
                        )),
                    });
                }
            }
        }

        if self.config.show_prs {
            let merged = fetch_all_pages(
                "https://api.github.com/search/issues?q=is:pr+is:merged+author:@me",
                &[],
                Some("items"),
            )?
            .len();
            let open = fetch_all_pages(
                "https://api.github.com/search/issues?q=is:pr+is:open+author:@me",
                &[],
                Some("items"),
            )?
            .len();
            let draft = fetch_all_pages(
                "https://api.github.com/search/issues?q=is:pr+is:draft+author:@me",
                &[],
                Some("items"),
            )?
            .len();
            if self.colour != "auto" {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("prs".to_string())
                        .apply_style(self.colour)
                        .bold(),
                    value: StyledText::new(format!(
                        "{} open | {} draft | {} merged",
                        open, draft, merged
                    )),
                });
            } else {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("prs".to_string()).bright_yellow().bold(),
                    value: StyledText::new(format!(
                        "{} open | {} draft | {} merged",
                        open, draft, merged
                    )),
                });
            }
        }

        if self.config.show_commits {
            let now = Utc::now();
            let today = now.format("%Y-%m-%d").to_string();
            let week_ago = (now - Duration::days(7)).format("%Y-%m-%d").to_string();
            let day =
                github_auth::make_request_with_token(github_auth::get_client()?.get(format!(
                    "https://api.github.com/search/commits?q=author:@me+committer-date:>={}",
                    today
                )))
                .context("Error getting recent commits")?
                .json::<serde_json::Value>()
                .context("Error parsing recent commits")?
                .get("total_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let week =
                github_auth::make_request_with_token(github_auth::get_client()?.get(format!(
                    "https://api.github.com/search/commits?q=author:@me+committer-date:>={}",
                    week_ago
                )))
                .context("Error getting recent commits")?
                .json::<serde_json::Value>()
                .context("Error parsing recent commits")?
                .get("total_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let all = github_auth::make_request_with_token(
                github_auth::get_client()?
                    .get("https://api.github.com/search/commits?q=author:@me"),
            )
            .context("Error getting recent commits")?
            .json::<serde_json::Value>()
            .context("Error parsing recent commits")?
            .get("total_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

            let day = if day >= 1000 {
                format!("{}k", day / 1000)
            } else {
                day.to_string()
            };
            let week = if week >= 1000 {
                format!("{}k", week / 1000)
            } else {
                week.to_string()
            };
            let all = if all >= 1000 {
                format!("{:.2}k", all as f64 / 1000.0)
                    .trim_end_matches("0")
                    .trim_end_matches(".")
                    .to_string()
            } else {
                all.to_string()
            };
            if self.colour != "auto" {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("commits".to_string())
                        .apply_style(self.colour)
                        .bold(),
                    value: StyledText::new(format!("{}d | {}w | {} total", day, week, all)),
                });
            } else {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("commits".to_string())
                        .bright_yellow()
                        .bold(),
                    value: StyledText::new(format!("{}d | {}w | {} total", day, week, all)),
                });
            }
        }

        if *self.show_section_gap {
            output.rows.push(OutputRow::Blank);
        }

        Ok(output)
    }
}
