use crate::component::Component;
use crate::output::{ComponentOutput, OutputRow};
use crate::styled_text::StyledText;
use anyhow::Result;
use git2::{Repository, StatusOptions};

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

        let cur_branch: String = repo
            .head()
            .ok()
            .and_then(|h| h.shorthand().ok().map(|s| s.to_string()))
            .unwrap_or_else(|| "Detatched HEAD".into());

        output.rows.push(OutputRow::Info {
            label: StyledText::new("current_branch").cyan().bold(),
            value: StyledText::new(cur_branch),
        });

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
