use crate::component::Component;
use crate::config::GithubConfig;
use crate::github_auth;
use crate::output::{ComponentOutput, OutputRow};
use crate::styled_text::StyledText;
use anyhow::{Context, Result};
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

        let username = match github_auth::username() {
            Ok(name) => name,
            Err(_) => return Err(anyhow::anyhow!("Failed to retrieve GitHub username")),
        };

        if self.colour != "auto" {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("username".to_string())
                    .apply_style(self.colour)
                    .bold(),
                value: StyledText::new(format!("@{}", username)),
            });
        } else {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("username".to_string()).yellow().bold(),
                value: StyledText::new(format!("@{}", username)),
            });
        }

        let (owner, repo) = if url.contains("git@github.com") {
            url.split(":")
                .nth(1)
                .and_then(|s| s.strip_suffix(".git"))
                .and_then(|s| s.split_once('/'))
                .map(|(o, r)| (o.to_string(), r.to_string()))
                .unwrap_or(("Unknown".to_string(), "Unknown".to_string()))
        } else if url.contains("github.com") {
            url.strip_prefix("https://github.com/")
                .and_then(|s| {
                    if s.contains(".git") {
                        s.strip_suffix(".git")
                    } else {
                        Some(s)
                    }
                })
                .and_then(|s| s.split_once('/'))
                .map(|(o, r)| (o.to_string(), r.to_string()))
                .unwrap_or(("Unknown".to_string(), "Unknown".to_string()))
        } else {
            return Err(anyhow::anyhow!("Failed to parse GitHub URL"));
        };

        if self.colour != "auto" {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("name".to_string())
                    .apply_style(self.colour)
                    .bold(),
                value: StyledText::new(format!("{}/{}", username, repo)),
            });
        } else {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("name".to_string()).bright_blue().bold(),
                value: StyledText::new(format!("{}/{}", username, repo)),
            });
        }

        let repo_info = github_auth::make_request_with_token(
            github_auth::get_client()?
                .get(format!("https://api.github.com/repos/{}/{}", owner, repo)),
        )
        .context("Failed to fetch repository information from GitHub")?;

        let pulls = github_auth::make_request_with_token(
            github_auth::get_client()?
                .get(format!(
                    "https://api.github.com/repos/{}/{}/pulls",
                    owner, repo
                ))
                .form(&[("state", "open"), ("per_page", "100")]),
        )
        .context("Failed to fetch pull requests information from GitHub")?;
        let open_pulls = pulls.len();
        let awaiting_review = pulls
            .iter()
            .filter(|pull| {
                pull.get("requested_reviewers")
                    .and_then(|rev| rev.as_array())
                    .is_some_and(|reviewers| {
                        reviewers
                            .iter()
                            .any(|rev| rev.get("login").and_then(|v| v.as_str() == Some(username)))
                    })
            })
            .count();
        let issues = github_auth::make_request_with_token(
            github_auth::get_client()?
                .get(format!(
                    "https://api.github.com/repos/{}/{}/issues",
                    owner, repo
                ))
                .form(&[("state", "open"), ("per_page", "100")]),
        )
        .context("Failed to fetch pull requests information from GitHub")?;
        let open_issues = issues.len();
        let awaiting_review = issues
            .iter()
            .filter(|pull| {
                pull.get("assignees")
                    .and_then(|rev| rev.as_array())
                    .is_some_and(|reviewers| {
                        reviewers
                            .iter()
                            .any(|rev| rev.get("login").and_then(|v| v.as_str() == Some(username)))
                    })
            })
            .count();
        let actions = github_auth::make_request_with_token(
            github_auth::get_client()?
            .get(format!("https://api.github.com/repos/{}/{}/commits/{}"))
        )

        if self.colour != "auto" {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("stars".to_string())
                    .apply_style(self.colour)
                    .bold(),
                value: StyledText::new(format!(
                    "{}",
                    repo_info
                        .get("stargazers_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                )),
            });
        } else {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("stars".to_string()).magenta().bold(),
                value: StyledText::new(format!(
                    "{}",
                    repo_info
                        .get("stargazers_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                )),
            });
        }
        if self.colour != "auto" {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("forks".to_string())
                    .apply_style(self.colour)
                    .bold(),
                value: StyledText::new(format!(
                    "{}",
                    repo_info
                        .get("forks_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                )),
            });
        } else {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("forks".to_string()).magenta().bold(),
                value: StyledText::new(format!(
                    "{}",
                    repo_info
                        .get("forks_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                )),
            });
        }
        if self.colour != "auto" {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("issues".to_string())
                    .apply_style(self.colour)
                    .bold(),
                value: StyledText::new(format!(
                    "{}",
                    repo_info
                        .get("open_issues_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                )),
            });
        } else {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("issues".to_string()).magenta().bold(),
                value: StyledText::new(format!(
                    "{}",
                    repo_info
                        .get("open_issues_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                )),
            });
        }
        if self.colour != "auto" {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("prs".to_string())
                    .apply_style(self::colour)
                    .bold(),
                value: StyledText::new(format!(
                    "{} open | {} awaiting review",
                    open_pulls, awaiting_review
                )),
            })
        } else {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("prs".to_string()).magenta().bold(),
                value: StyledText::new(format!(
                    "{} open | {} awaiting review",
                    open_pulls, awaiting_review
                )),
            })
        }

        Ok(output)
    }
}
