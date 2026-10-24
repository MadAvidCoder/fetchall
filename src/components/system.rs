use crate::component::Component;
use crate::output::{ComponentOutput, OutputRow};
use crate::styled_text::StyledText;
use anyhow::Result;
use owo_colors::OwoColorize;
use sysinfo::{Disks, System};

pub struct SystemComponent;

impl Component for SystemComponent {
    fn title(&self) -> &str {
        "System"
    }

    fn collect(&self) -> Result<ComponentOutput> {
        let os_name: String = System::name().unwrap_or_else(|| "Unknown".to_string());

        if sysinfo::IS_SUPPORTED_SYSTEM {
            let mut info: ComponentOutput = ComponentOutput { rows: Vec::new() };

            let mut sys = System::new_all();
            let disks = Disks::new_with_refreshed_list();

            let mut space: u64 = 0;
            let mut available_space: u64 = 0;

            for disk in disks.list() {
                space += disk.total_space();
                available_space += disk.available_space();
            }

            sys.refresh_all();

            let used_mem = sys.used_memory() as f64 / 1_073_741_824.0;
            let total_mem = sys.total_memory() as f64 / 1_073_741_824.0;

            let used_disk = (space - available_space) as f64 / 1_073_741_824.0;
            let total_disk = space as f64 / 1_073_741_824.0;
            let free_disk = available_space as f64 / 1_073_741_824.0;

            info.rows.push(OutputRow::Section(
                StyledText::new("System Information").bold().underline(),
            ));

            // cpu
            let cpu_usage = sys.global_cpu_usage();
            let cpu_name = sys
                .cpus()
                .first()
                .map(|c| c.brand())
                .unwrap_or("Unknown CPU");

            // println!("  {} {}", "os   :".blue().bold(), os_name.white());
            info.rows.push(OutputRow::Info {
                label: StyledText::new("os").blue().bold(),
                value: StyledText::new(os_name),
            });

            // println!(
            //     "  {} {:.2} / {:.2} GB",
            //     "mem  :".green().bold(),
            //     used_mem,
            //     total_mem
            // );
            info.rows.push(OutputRow::Info {
                label: StyledText::new("mem").green().bold(),
                value: StyledText::new(format!("{:.2} / {:.2} GB", used_mem, total_mem)),
            });

            // println!(
            //     "  {} {:.2} / {:.2} GB {} {:.2} {}",
            //     "disk :".yellow().bold(),
            //     used_disk,
            //     total_disk,
            //     "(free:".dimmed(),
            //     free_disk.dimmed(),
            //     "GB)".dimmed(),
            // );
            info.rows.push(OutputRow::Info {
                label: StyledText::new("disk").yellow().bold(),
                value: StyledText::new(format!(
                    "{:.2} / {:.2} GB (free: {:.2} GB)",
                    used_disk, total_disk, free_disk
                )),
            });

            // println!(
            //     "  {} {} ({:.1}%)",
            //     "cpu  :".red().bold(),
            //     cpu_name,
            //     cpu_usage
            // );
            info.rows.push(OutputRow::Info {
                label: StyledText::new("cpu").red().bold(),
                value: StyledText::new(format!("{} ({:.1}%)", cpu_name, cpu_usage)),
            });

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
