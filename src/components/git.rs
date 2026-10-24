use crate::component::Component;
use crate::output::{ComponentOutput, OutputRow};
use crate::styled_text::StyledText;
use anyhow::Result;
use git2::{BranchType, Repository, StatusOptions};

pub struct GitComponent;

impl Component for GitComponent {
    fn title(&self) -> &str {
        "Git Insights"
    }

    fn collect(&self) -> Result<ComponentOutput> {
        let mut output = ComponentOutput { rows: Vec::new() };
        output.rows.push(OutputRow::Section(
            StyledText::new(self.title()).bold().underline(),
        ));
        let repo = match Repository::discover(".") {
            Ok(repo) => repo,
            Err(_) => {
                return Err(anyhow::anyhow!("Failed to discover Git repository"));
            }
        };

        let root = repo.workdir().unwrap_or_else(|| repo.path());
        let root_path = root.display();
        let root_name = root
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"))
            .display();
        output.rows.push(OutputRow::Info {
            label: StyledText::new("repo").blue().bold(),
            value: StyledText::new(format!("{} ({})", root_name, root_path)),
        });

        let head = repo.head().ok();

        let cur_branch = head
            .as_ref()
            .and_then(|h| h.shorthand().ok().map(str::to_owned));
        let cur_branch_text = cur_branch.as_deref().unwrap_or("Detached HEAD");
        
        if let Some(cur_branch) = cur_branch.as_deref()
            && let Ok(branch) = repo.find_branch(cur_branch, BranchType::Local)
            && let Ok(upstream) = branch.upstream()
            && let Some(head) = head
            && let Ok((ahead, behind)) = repo.graph_ahead_behind(
                head.peel_to_commit()?.id(),
                upstream.get().peel_to_commit()?.id(),
            )
        {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("branch").cyan().bold(),
                value: StyledText::new(format!("{cur_branch_text} ↑{ahead} ↓{behind}")),
            });
        } else {
            output.rows.push(OutputRow::Info {
                label: StyledText::new("branch").cyan().bold(),
                value: StyledText::new(cur_branch_text),
            });
        }

        let statuses = repo.statuses(Some(
            StatusOptions::new()
                .include_untracked(true)
                .recurse_untracked_dirs(true),
        ))?;
        let untracked = statuses
            .iter()
            .filter(|entry| entry.status().is_wt_new())
            .count();
        let staged = statuses
            .iter()
            .filter(|entry| entry.status().is_index_new() || entry.status().is_index_modified())
            .count();
        let unstaged = statuses
            .iter()
            .filter(|entry| entry.status().is_wt_modified() || entry.status().is_wt_deleted())
            .count();

        output.rows.push(OutputRow::Info {
            label: StyledText::new("changes").yellow().bold(),
            value: StyledText::new(if untracked + staged + unstaged == 0 {
                "Working tree clean".into()
            } else {
                [
                    (staged, "staged"),
                    (unstaged, "unstaged"),
                    (untracked, "untracked"),
                ]
                .into_iter()
                .filter(|(count, _)| *count > 0)
                .map(|(count, label)| format!("{} {}", count, label))
                .collect::<Vec<String>>()
                .join(" | ")
            }),
        });

        Ok(output)
    }
}
