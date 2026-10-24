mod component;
mod components;
mod helpers;
mod output;
mod renderer;
mod styled_text;

use crate::component::Component;
use crate::components::git::GitComponent;
use crate::components::system::SystemComponent;
use crate::renderer::render;
use anyhow::Result;
use clap::Parser;

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
    #[arg(short, long, help = "Test")]
    test: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.test {
        println!("Hello, world!");
        return Ok(());
    }
    let git_info = GitComponent.collect()?;
    let system_info = SystemComponent.collect()?;

    render(&(system_info + git_info));

    Ok(())
}
