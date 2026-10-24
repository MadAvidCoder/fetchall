use clap::Parser;
use indoc::indoc;
use itertools::{EitherOrBoth, Itertools};
use owo_colors::{OwoColorize, Style};
use sysinfo::{Disks, System};
use terminal_size::{Height, Width, terminal_size};
use unicode_width::UnicodeWidthStr;

macro_rules! style_methods {
    ($($name:ident),* $(,)?) => {
        $(
            #[allow(dead_code)]
            pub fn $name(mut self) -> Self {
                self.style = self.style.$name();
                self
            }
        )*
    };
}

struct StyledText {
    text: String,
    style: Style,
}

impl StyledText {
    fn new(text: impl Into<String>) -> Self {
        StyledText {
            text: text.into(),
            style: Style::new(),
        }
    }

    style_methods!(
        black,
        red,
        green,
        yellow,
        blue,
        magenta,
        cyan,
        white,
        bright_black,
        bright_red,
        bright_green,
        bright_yellow,
        bright_blue,
        bright_magenta,
        bright_cyan,
        bright_white,
        on_black,
        on_red,
        on_green,
        on_yellow,
        on_blue,
        on_magenta,
        on_cyan,
        on_white,
        bold,
        dimmed,
        italic,
        underline,
        strikethrough,
        reversed,
    );

    fn lines(&self) -> impl Iterator<Item = Self> + '_ {
        self.text.lines().map(|line| Self {
            text: line.to_string(),
            style: self.style,
        })
    }

    fn width(&self) -> usize {
        UnicodeWidthStr::width(&self.text as &str)
    }

    fn render(&self) -> String {
        self.text.style(self.style).to_string()
    }
}

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
        let mut info: Vec<(StyledText, StyledText)> = Vec::new();

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
        info.push((
            StyledText::new("os").blue().bold(),
            StyledText::new(os_name),
        ));

        // println!(
        //     "  {} {:.2} / {:.2} GB",
        //     "mem  :".green().bold(),
        //     used_mem,
        //     total_mem
        // );
        info.push((
            StyledText::new("mem").green().bold(),
            StyledText::new(format!("{:.2} / {:.2} GB", used_mem, total_mem)),
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
            StyledText::new("disk").yellow().bold(),
            StyledText::new(format!(
                "{:.2} / {:.2} GB (free: {:.2} GB)",
                used_disk, total_disk, free_disk
            )),
        ));

        // println!(
        //     "  {} {} ({:.1}%)",
        //     "cpu  :".red().bold(),
        //     cpu_name,
        //     cpu_usage
        // );
        info.push((
            StyledText::new("cpu").red().bold(),
            StyledText::new(format!("{} ({:.1}%)", cpu_name, cpu_usage)),
        ));

        println!();

        let logo = StyledText::new(indoc! {r#"
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
        "#})
        .bright_blue();

        let logo_gap: usize = 4;
        let logo_width = logo.lines().map(|line| line.width()).max().unwrap_or(40);

        let label_gap: usize = 2;
        let label_width = info
            .iter()
            .map(|(label, _)| label.width())
            .max()
            .unwrap_or(10);

        for paired in logo.lines().zip_longest(info.iter()) {
            let (line, (label, value)) = match paired {
                EitherOrBoth::Both(line, (label, value)) => (line, (label, value)),
                EitherOrBoth::Left(line) => (line, (&StyledText::new(""), &StyledText::new(""))),
                EitherOrBoth::Right((label, value)) => (StyledText::new(""), (label, value)),
            };
            let logo_pad: usize = if logo_width > line.width() {
                logo_width - line.width() + logo_gap
            } else {
                logo_gap
            };
            let label_pad: usize = if label_width > label.width() {
                label_width - label.width() + label_gap - 1
            } else {
                label_gap - 1
            };
            println!(
                "{}{}{}{} {} {}",
                line.render(),
                " ".repeat(logo_pad),
                label.render(),
                " ".repeat(label_pad),
                if label.text.is_empty() { " " } else { ":" },
                value.render(),
            );
        }
    } else {
        println!("{}", "This OS isn't supported".red());
    }
}
