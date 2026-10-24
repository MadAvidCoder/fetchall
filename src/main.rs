use clap::Parser;
use indoc::indoc;
use itertools::{EitherOrBoth, Itertools};
use owo_colors::OwoColorize;
use sysinfo::{Disks, System};
use terminal_size::{Height, Width, terminal_size};
use unicode_width::UnicodeWidthStr;

#[derive(Parser)]
#[command(
    name="fetchall",
    // TODO: add about
    about="TBA",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_subcommand = true,
    author = "MadAvidCoder"
)]
struct Args {
    #[arg(short, long, help = "Show version")]
    version: bool,
}

fn main() {
    let (width, height) = terminal_size()
        .map(|(Width(w), Height(h))| (w, h))
        .unwrap_or((80, 24));

    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());

    if sysinfo::IS_SUPPORTED_SYSTEM {
        let mut info: Vec<(String, String)> = Vec::new();

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

        // cpu
        let cpu_usage = sys.global_cpu_usage();
        let cpu_name = sys
            .cpus()
            .first()
            .map(|c| c.brand())
            .unwrap_or("Unknown CPU");

        println!();
        println!("{}", "  minimalfetch".bold().cyan());
        println!();

        // println!("  {} {}", "os   :".blue().bold(), os_name.white());
        info.push(("os".to_string(), os_name.clone()));

        // println!(
        //     "  {} {:.2} / {:.2} GB",
        //     "mem  :".green().bold(),
        //     used_mem,
        //     total_mem
        // );
        info.push((
            "mem".to_string(),
            format!("{:.2} / {:.2} GB", used_mem, total_mem),
        ));

        // println!(
        //     "  {} {:.2} / {:.2} GB {} {:.2} {}",
        //     "disk :".yellow().bold(),
        //     used_disk,
        //     total_disk,
        //     "(free:".dimmed(),
        //     free_disk.dimmed(),
        //     "GB)".dimmed(),
        // );
        info.push((
            "disk".to_string(),
            format!(
                "{:.2} / {:.2} GB (free: {:.2} GB)",
                used_disk, total_disk, free_disk
            ),
        ));

        // println!(
        //     "  {} {} ({:.1}%)",
        //     "cpu  :".red().bold(),
        //     cpu_name,
        //     cpu_usage
        // );
        info.push((
            "cpu".to_string(),
            format!("{} ({:.1}%)", cpu_name, cpu_usage),
        ));

        println!();

        let logo: &str = indoc! {r#"
            ///////////////// /////////////////
            ///////////////// /////////////////
            ///////////////// /////////////////
            ///////////////// /////////////////
            ///////////////// /////////////////
            ///////////////// /////////////////
            ///////////////// /////////////////
            ///////////////// /////////////////

            ///////////////// /////////////////
            ///////////////// /////////////////
            ///////////////// /////////////////
            ///////////////// /////////////////
            ///////////////// /////////////////
            ///////////////// /////////////////
            ///////////////// /////////////////
            ///////////////// /////////////////
        "#};

        let logo_gap: usize = 4;
        let logo_width = logo.lines().map(UnicodeWidthStr::width).max().unwrap_or(40);

        let label_gap: usize = 2;
        let label_width = info
            .iter()
            .map(|(label, _)| UnicodeWidthStr::width(label.as_str()))
            .max()
            .unwrap_or(10);

        for paired in logo.lines().zip_longest(info.iter()) {
            let (line, (label, value)) = match paired {
                EitherOrBoth::Both(line, (label, value)) => (line, (label, value)),
                EitherOrBoth::Left(line) => (line, (&"".to_string(), &"".to_string())),
                EitherOrBoth::Right((label, value)) => ("", (label, value)),
            };
            let logo_pad: usize = if logo_width > line.len() {
                logo_width - line.len() + logo_gap
            } else {
                logo_gap
            };
            println!(
                "{}{}{:<label_pad$} {} {}",
                line,
                " ".repeat(logo_pad),
                label,
                if label.is_empty() { " " } else { ":" },
                value,
                label_pad = label_width + label_gap - 1,
            );
        }
    } else {
        println!("{}", "This OS isn't supported".red());
    }
}
