use crate::component::Component;
use crate::config::GithubRepoConfig;
use crate::github_auth;
use crate::output::{ComponentOutput, OutputRow};
use crate::styled_text::StyledText;
use anyhow::{Context, Result};
use git2::Repository;
use reqwest::header::LINK;

pub fn fetch_all_pages(
    url: &str,
    query: &[(&str, &str)],
    field: Option<&str>,
) -> Result<Vec<serde_json::Value>> {
    let mut next_url = Some(url.to_string());
    let client = github_auth::get_client()?;
    let mut results = Vec::new();

    while let Some(url) = next_url {
        let response = github_auth::make_request_with_token(
            client.get(&url).query(query).query(&[("per_page", "100")]),
        )
        .context("Failed to get page")?;
        let link = response
            .headers()
            .get(LINK)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let body: Vec<serde_json::Value> = match field {
            Some(f) => response
                .json::<serde_json::Value>()
                .context("Failed to parse JSON")?
                .get(f)
                .context("Expected field not found")?
                .as_array()
                .context("Expected JSON array")?
                .to_vec(),
            None => response
                .json::<serde_json::Value>()
                .context("Failed to parse JSON")?
                .as_array()
                .context("Expected JSON array")?
                .to_vec(),
        };
        results.extend(body.iter().cloned());
        next_url = link.as_deref().and_then(|link| {
            link.split(',').find_map(|part| {
                let part = part.trim();
                if part.ends_with("rel=\"next\"") {
                    part.split(';')
                        .next()
                        .map(|url_part| {
                            url_part
                                .trim()
                                .trim_start_matches('<')
                                .trim_end_matches('>')
                        })
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
        });
    }
    Ok(results)
}

pub struct GithubRepoComponent<'a> {
    config: &'a GithubRepoConfig,
    colour: &'a str,
    show_section_titles: &'a bool,
}

impl<'a> GithubRepoComponent<'a> {
    pub fn new(
        config: &'a GithubRepoConfig,
        colour: &'a str,
        show_section_titles: &'a bool,
    ) -> Self {
        Self {
            config,
            colour,
            show_section_titles,
        }
    }
}

impl Component for GithubRepoComponent<'_> {
    fn title(&self) -> &str {
        "GitHub Repo Info"
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

        if self.config.show_url {
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
        }

        let username = match github_auth::username() {
            Ok(name) => name,
            Err(e) => return Err(anyhow::anyhow!("Failed to retrieve GitHub username: {}", e)),
        };

        let (owner, repo) = if url.contains("git@github.com") {
            url.split(":")
                .nth(1)
                .and_then(|s| s.strip_suffix(".git"))
                .and_then(|s| s.split_once('/'))
                .map(|(o, r)| (o.to_string(), r.to_string()))
                .unwrap_or(("Unknown".to_string(), "Unknown".to_string()))
        } else if url.contains("://github.com") {
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

        if self.config.show_repo {
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
        }

        let repo_info = github_auth::make_request_with_token(
            github_auth::get_client()?
                .get(format!("https://api.github.com/repos/{}/{}", owner, repo)),
        )
        .context("Failed to fetch repository information from GitHub")?
        .json::<serde_json::Value>()
        .context("Failed to parse JSON")?;

        if self.config.show_stars {
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
        }
        if self.config.show_forks {
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
        }
        if self.config.show_issues {
            if self.colour != "auto" {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("issues".to_string())
                        .apply_style(self.colour)
                        .bold(),
                    value: StyledText::new(format!(
                        "{} assigned | {} open ",
                        issues_waiting, open_issues
                    )),
                });
            } else {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("issues".to_string()).magenta().bold(),
                    value: StyledText::new(format!(
                        "{} assigned | {} open",
                        issues_waiting, open_issues
                    )),
                });
            }
        }

        if self.config.show_prs {
            let pulls = fetch_all_pages(
                &format!("https://api.github.com/repos/{}/{}/pulls", owner, repo),
                &[("state", "open")],
                None,
            )
            .context("Failed to fetch pull requests information from GitHub")?;
            let open_pulls = pulls.len();
            let prs_waiting = pulls
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
            let issues = fetch_all_pages(
                &format!("https://api.github.com/repos/{}/{}/issues", owner, repo),
                &[("state", "open")],
                None,
            )
            .context("Failed to fetch pull requests information from GitHub")?;
            let open_issues = issues.len();
            let issues_waiting = issues
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
            if self.colour != "auto" {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("prs".to_string())
                        .apply_style(self.colour)
                        .bold(),
                    value: StyledText::new(format!(
                        "{} awaiting review | {} open",
                        prs_waiting, open_pulls
                    )),
                })
            } else {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("prs".to_string()).magenta().bold(),
                    value: StyledText::new(format!(
                        "{} awaiting review | {} open",
                        prs_waiting, open_pulls
                    )),
                })
            }
        }
        if self.config.show_actions {
            let latest_sha = github_auth::make_request_with_token(
                github_auth::get_client()?
                    .get(format!(
                        "https://api.github.com/repos/{}/{}/commits",
                        owner, repo
                    ))
                    .query(&[("per_page", "1")]),
            )
            .context("Failed to fetch latest commit information from GitHub")?
            .json::<serde_json::Value>()
            .context("Failed to fetch latest commit information from GitHub")?
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|commit| commit.get("sha"))
            .and_then(|sha| sha.as_str())
            .unwrap_or("Unknown")
            .to_string();

            let actions = fetch_all_pages(
                &format!(
                    "https://api.github.com/repos/{}/{}/commits/{}/check-runs",
                    owner, repo, latest_sha
                ),
                &[],
                Some("check_runs"),
            )
            .context("Failed to fetch GitHub Actions information from GitHub")?;

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
                            complete + running
                        )),
                    });
                } else if failed > 0 {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("actions".to_string()).bright_blue().bold(),
                        value: StyledText::new(format!(
                            "✗ {}/{} failed",
                            failed,
                            complete + failed
                        )),
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
        }

        output.rows.push(OutputRow::Blank);

        Ok(output)
    }
}
