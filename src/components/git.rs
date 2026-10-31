use crate::component::Component;
use crate::config::GitConfig;
use crate::helpers::format_relative_time;
use crate::output::{ComponentOutput, OutputRow};
use crate::styled_text::StyledText;
use anyhow::Result;
use git2::{BranchType, Repository, Status, StatusOptions};

pub struct GitComponent<'a> {
    config: &'a GitConfig,
    colour: &'a str,
    show_section_titles: &'a bool,
    show_section_gap: &'a bool,
}

impl<'a> GitComponent<'a> {
    pub fn new(
        config: &'a GitConfig,
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

impl Component for GitComponent<'_> {
    fn title(&self) -> &str {
        "Git Insights"
    }

    fn collect(&self) -> Result<ComponentOutput> {
        let mut output = ComponentOutput { rows: Vec::new() };

        if !self.config.enabled {
            return Ok(output);
        }
        if *self.show_section_titles {
            if self.colour != "auto" {
                output.rows.push(OutputRow::Section(
                    StyledText::new(self.title())
                        .apply_style(self.colour)
                        .bold()
                        .underline(),
                ));
            } else {
                output.rows.push(OutputRow::Section(
                    StyledText::new(self.title()).bold().underline(),
                ));
            }
        }

        if self.config.show_repo
            || self.config.show_branch
            || self.config.show_changes
            || self.config.show_commit
        {
            let repo = match Repository::discover(".") {
                Ok(repo) => repo,
                Err(_) => {
                    return Err(anyhow::anyhow!("Failed to discover Git repository"));
                }
            };

            if self.config.show_repo {
                let root = repo.workdir().unwrap_or_else(|| repo.path());
                let root_path = root.display();
                let root_name = root
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"))
                    .display();
                if self.colour != "auto" {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("repo").apply_style(self.colour).bold(),
                        value: StyledText::new(format!("{} ({})", root_name, root_path)),
                    });
                } else {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("repo").blue().bold(),
                        value: StyledText::new(format!("{} ({})", root_name, root_path)),
                    });
                }
            }

            if self.config.show_branch {
                let head = repo.head().ok();

                let cur_branch = head
                    .as_ref()
                    .and_then(|h| h.shorthand().ok().map(str::to_owned));
                let cur_branch_text = cur_branch.as_deref().unwrap_or("Detached HEAD");

                if let Some(cur_branch) = cur_branch.as_deref()
                    && let Ok(branch) = repo.find_branch(cur_branch, BranchType::Local)
                    && let Ok(upstream) = branch.upstream()
                    && let Some(ref head) = head
                    && let Ok((ahead, behind)) = repo.graph_ahead_behind(
                        head.peel_to_commit()?.id(),
                        upstream.get().peel_to_commit()?.id(),
                    )
                {
                    if self.colour != "auto" {
                        output.rows.push(OutputRow::Info {
                            label: StyledText::new("branch").apply_style(self.colour).bold(),
                            value: StyledText::new(format!("{cur_branch_text} ↑{ahead} ↓{behind}")),
                        });
                    } else {
                        output.rows.push(OutputRow::Info {
                            label: StyledText::new("branch").cyan().bold(),
                            value: StyledText::new(format!("{cur_branch_text} ↑{ahead} ↓{behind}")),
                        });
                    }
                } else {
                    if self.colour != "auto" {
                        output.rows.push(OutputRow::Info {
                            label: StyledText::new("branch").apply_style(self.colour).bold(),
                            value: StyledText::new(cur_branch_text),
                        });
                    } else {
                        output.rows.push(OutputRow::Info {
                            label: StyledText::new("branch").cyan().bold(),
                            value: StyledText::new(cur_branch_text),
                        });
                    }
                }
            }
            if self.config.show_changes {
                let statuses = repo.statuses(Some(
                    StatusOptions::new()
                        .include_untracked(true)
                        .recurse_untracked_dirs(true),
                ))?;

                let mut conflicts: usize = 0;
                let mut untracked: usize = 0;
                let mut staged: usize = 0;
                let mut unstaged: usize = 0;
                for entry in statuses.iter() {
                    let status = entry.status();
                    if status.is_conflicted() {
                        conflicts += 1;
                    }
                    if status.intersects(
                        Status::INDEX_NEW
                            | Status::INDEX_MODIFIED
                            | Status::INDEX_DELETED
                            | Status::INDEX_RENAMED
                            | Status::INDEX_TYPECHANGE,
                    ) {
                        staged += 1;
                    }
                    if status.intersects(
                        Status::WT_MODIFIED
                            | Status::WT_DELETED
                            | Status::WT_RENAMED
                            | Status::WT_TYPECHANGE,
                    ) {
                        unstaged += 1;
                    }
                    if status.intersects(Status::WT_NEW) {
                        untracked += 1;
                    }
                }
                if self.colour != "auto" {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("changes").apply_style(self.colour).bold(),
                        value: StyledText::new(if untracked + staged + unstaged == 0 {
                            "Working tree clean".into()
                        } else {
                            [
                                (staged, "staged"),
                                (unstaged, "modified"),
                                (untracked, "untracked"),
                            ]
                            .into_iter()
                            .filter(|(count, _)| *count > 0)
                            .map(|(count, label)| format!("{} {}", count, label))
                            .collect::<Vec<String>>()
                            .join(" | ")
                        }),
                    });
                } else {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("changes").yellow().bold(),
                        value: StyledText::new(if untracked + staged + unstaged == 0 {
                            "Working tree clean".into()
                        } else {
                            [
                                (staged, "staged"),
                                (unstaged, "modified"),
                                (untracked, "untracked"),
                            ]
                            .into_iter()
                            .filter(|(count, _)| *count > 0)
                            .map(|(count, label)| format!("{} {}", count, label))
                            .collect::<Vec<String>>()
                            .join(" | ")
                        }),
                    });
                }

                if conflicts > 0 {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("conflicts").red().bold(),
                        value: StyledText::new(format!("{conflicts} detected")),
                    });
                }
            }

            if self.config.show_commit {
                let latest_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
                let commit_value = match latest_commit {
                    Some(commit) => {
                        let hash = commit.id().to_string();
                        let short_hash = hash[..7.min(hash.len())].to_string();
                        let summary = commit.message().ok().and_then(|s| s.split("\n").next());
                        let time = format_relative_time(commit.time().seconds());
                        if let Some(summary) = summary {
                            format!("{} | {} | {}", short_hash, time, summary)
                        } else {
                            format!("{} | {}", short_hash, time)
                        }
                    }
                    None => "No commits yet".into(),
                };
                if self.colour != "auto" {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("latest").apply_style(self.colour).bold(),
                        value: StyledText::new(commit_value),
                    });
                } else {
                    output.rows.push(OutputRow::Info {
                        label: StyledText::new("latest").magenta().bold(),
                        value: StyledText::new(commit_value),
                    });
                }
            }
        }

        if *self.show_section_gap {
            output.rows.push(OutputRow::Blank);
        }

        Ok(output)
    }
}
