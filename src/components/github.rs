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
            Err(e) => return Err(anyhow::anyhow!("Failed to retrieve GitHub username: {}", e)),
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
        let open_pulls = pulls.as_array().map(|v| v.len()).unwrap_or(0);
        let prs_waiting = pulls
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter(|pull| {
                pull.get("requested_reviewers")
                    .and_then(|rev| rev.as_array())
                    .is_some_and(|reviewers| {
                        reviewers.iter().any(|rev| {
                            rev.get("login")
                                .unwrap_or(&serde_json::Value::String("".into()))
                                .as_str()
                                .unwrap_or("")
                                == username.as_str()
                        })
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
        let open_issues = issues.as_array().map(|v| v.len()).unwrap_or(0);
        let issues_waiting = issues
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter(|pull| {
                pull.get("assignees")
                    .and_then(|rev| rev.as_array())
                    .is_some_and(|reviewers| {
                        reviewers.iter().any(|rev| {
                            rev.get("login")
                                .unwrap_or(&serde_json::Value::String("".into()))
                                .as_str()
                                .unwrap_or("")
                                == username.as_str()
                        })
                    })
            })
            .count();
        let latest_sha = github_auth::make_request_with_token(
            github_auth::get_client()?
                .get(format!(
                    "https://api.github.com/repos/{}/{}/commits",
                    owner, repo
                ))
                .form(&[("per_page", "1")]),
        )
        .context("Failed to fetch latest commit information from GitHub")?
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|commit| commit.get("sha"))
        .and_then(|sha| sha.as_str())
        .unwrap_or("Unknown")
        .to_string();

        let actions = github_auth::make_request_with_token(
            github_auth::get_client()?
                .get(format!(
                    "https://api.github.com/repos/{}/{}/commits/{}/check-runs",
                    owner, repo, latest_sha
                ))
                .form(&[("per_page", "100")]),
        )
        .context("Failed to fetch GitHub Actions information from GitHub")?
        .get("check_runs")
        .unwrap_or(&serde_json::Value::Null)
        .as_array()
        .unwrap_or(&Vec::new())
        .to_vec();
        println!(
            "Actions: {:?}",
            actions
                .iter()
                .map(|c| c.get("conclusion"))
                .collect::<Vec<_>>()
        );

        let complete = actions
            .iter()
            .filter(|c| {
                matches!(
                    c.get("status")
                        .and_then(|conclusion| conclusion.as_str())
                        .unwrap_or(""),
                    "completed"
                )
            })
            .count();

        let running = actions
            .iter()
            .filter(|c| {
                c.get("status")
                    .and_then(|status| status.as_str())
                    .unwrap_or("")
                    != "completed"
            })
            .count();

        let failed = actions
            .iter()
            .filter(|c| {
                matches!(
                    c.get("conclusion")
                        .and_then(|conclusion| conclusion.as_str())
                        .unwrap_or(""),
                    "failure" | "timed_out" | "cancelled" | "action_required"
                )
            })
            .count();

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
                    "{} open | {} assigned ",
                    open_issues, issues_waiting
                )),
            });
        } else {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("issues".to_string()).magenta().bold(),
                value: StyledText::new(format!(
                    "{} open | {} assigned",
                    open_issues, issues_waiting
                )),
            });
        }
        if self.colour != "auto" {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("prs".to_string())
                    .apply_style(self.colour)
                    .bold(),
                value: StyledText::new(format!(
                    "{} open | {} awaiting review",
                    open_pulls, prs_waiting
                )),
            })
        } else {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("prs".to_string()).magenta().bold(),
                value: StyledText::new(format!(
                    "{} open | {} awaiting review",
                    open_pulls, prs_waiting
                )),
            })
        }
        if self.colour != "auto" {
            if running > 0 {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("actions".to_string())
                        .apply_style(self.colour)
                        .bold(),
                    value: StyledText::new(format!(
                        "◔ {}/{} completed",
                        complete,
                        complete + running
                    )),
                });
            } else if failed > 0 {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("actions".to_string())
                        .apply_style(self.colour)
                        .bold(),
                    value: StyledText::new(format!("✗ {}/{} failed", failed, complete)),
                });
            } else if complete > 0 {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("actions".to_string())
                        .apply_style(self.colour)
                        .bold(),
                    value: StyledText::new(format!("✓ All {}/{} passing", complete, complete)),
                });
            } else {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("actions".to_string())
                        .apply_style(self.colour)
                        .bold(),
                    value: StyledText::new("No actions found"),
                });
            }
        } else {
            if running > 0 {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("actions".to_string()).bright_blue().bold(),
                    value: StyledText::new(format!(
                        "◔ {}/{} completed",
                        running,
                        complete + failed
                    )),
                });
            } else if failed > 0 {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("actions".to_string()).bright_blue().bold(),
                    value: StyledText::new(format!("✗ {}/{} failed", failed, complete + failed)),
                });
            } else if complete > 0 {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("actions".to_string()).bright_blue().bold(),
                    value: StyledText::new(format!("✓ All {}/{} passing", complete, complete)),
                });
            } else {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("actions".to_string()).bright_blue().bold(),
                    value: StyledText::new("No actions found"),
                });
            }
        }

        output.rows.push(OutputRow::Blank);

        Ok(output)
    }
}
