use crate::component::Component;
use crate::config::SystemConfig;
use crate::output::{ComponentOutput, OutputRow};
use crate::styled_text::StyledText;
use anyhow::Result;
use owo_colors::OwoColorize;
use sysinfo::{Disks, System};

pub struct SystemComponent<'a> {
    config: &'a SystemConfig,
    colour: &'a str,
    show_section_titles: &'a bool,
}

impl<'a> SystemComponent<'a> {
    pub fn new(config: &'a SystemConfig, colour: &'a str, show_section_titles: &'a bool) -> Self {
        Self {
            config,
            colour,
            show_section_titles,
        }
    }
}

impl Component for SystemComponent<'_> {
    fn title(&self) -> &str {
        "System"
    }

    fn collect(&self) -> Result<ComponentOutput> {
        if !self.config.enabled {
            return Ok(ComponentOutput { rows: Vec::new() });
        }
        let os_name: String = System::name().unwrap_or_else(|| "Unknown".to_string());

        if sysinfo::IS_SUPPORTED_SYSTEM {
            let mut info: ComponentOutput = ComponentOutput { rows: Vec::new() };
            if *self.show_section_titles {
                info.rows.push(OutputRow::Section(
                    StyledText::new("System Information").bold().underline(),
                ));
            }

            if self.config.show_os {
                if self.colour != "auto" {
                    info.rows.push(OutputRow::Info {
                        label: StyledText::new("os").blue().bold(),
                        value: StyledText::new(&os_name).apply_style(self.colour).bold(),
                    });
                } else {
                    info.rows.push(OutputRow::Info {
                        label: StyledText::new("os").blue().bold(),
                        value: StyledText::new(&os_name),
                    });
                }
            }

            let mut sys = System::new_all();

            sys.refresh_all();

            if self.config.show_memory {
                let used_mem = sys.used_memory() as f64 / 1_073_741_824.0;
                let total_mem = sys.total_memory() as f64 / 1_073_741_824.0;

                if self.colour != "auto" {
                    info.rows.push(OutputRow::Info {
                        label: StyledText::new("mem").apply_style(self.colour).bold(),
                        value: StyledText::new(format!("{:.2} / {:.2} GB", used_mem, total_mem))
                            .apply_style(self.colour),
                    });
                } else {
                    info.rows.push(OutputRow::Info {
                        label: StyledText::new("mem").green().bold(),
                        value: StyledText::new(format!("{:.2} / {:.2} GB", used_mem, total_mem)),
                    });
                }
            }

            if self.config.show_disk {
                let disks = Disks::new_with_refreshed_list();
                let mut space: u64 = 0;
                let mut available_space: u64 = 0;

                for disk in disks.list() {
                    space += disk.total_space();
                    available_space += disk.available_space();
                }

                let used_disk = (space - available_space) as f64 / 1_073_741_824.0;
                let total_disk = space as f64 / 1_073_741_824.0;
                let free_disk = available_space as f64 / 1_073_741_824.0;

                if self.colour != "auto" {
                    info.rows.push(OutputRow::Info {
                        label: StyledText::new("disk").apply_style(self.colour).bold(),
                        value: StyledText::new(format!(
                            "{:.2} / {:.2} GB (free: {:.2} GB)",
                            used_disk, total_disk, free_disk
                        ))
                        .apply_style(self.colour),
                    });
                } else {
                    info.rows.push(OutputRow::Info {
                        label: StyledText::new("disk").yellow().bold(),
                        value: StyledText::new(format!(
                            "{:.2} / {:.2} GB (free: {:.2} GB)",
                            used_disk, total_disk, free_disk
                        )),
                    });
                }
            }

            if self.config.show_cpu {
                let cpu_usage = sys.global_cpu_usage();
                let cpu_name = sys
                    .cpus()
                    .first()
                    .map(|c| c.brand())
                    .unwrap_or("Unknown CPU");

                if self.colour != "auto" {
                    info.rows.push(OutputRow::Info {
                        label: StyledText::new("cpu").apply_style(self.colour).bold(),
                        value: StyledText::new(format!("{} ({:.1}%)", cpu_name, cpu_usage))
                            .apply_style(self.colour),
                    });
                } else {
                    info.rows.push(OutputRow::Info {
                        label: StyledText::new("cpu").red().bold(),
                        value: StyledText::new(format!("{} ({:.1}%)", cpu_name, cpu_usage)),
                    });
                }
            }

            info.rows.push(OutputRow::Blank);

            Ok(info)
        } else {
            Err(anyhow::Error::msg(format!(
                "{}",
                "This OS isn't supported".red()
            )))
        }
    }
}
