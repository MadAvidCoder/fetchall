mod component;
mod components;
mod helpers;
mod output;
mod renderer;
mod styled_text;

use crate::component::Component;
use crate::components::git::GitComponent;
use crate::components::system::SystemComponent;
use crate::components::user::UserComponent;
use crate::renderer::render;
use anyhow::Result;
use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, ValueEnum, Debug)]
enum Profile {
    Default,
    System,
    Dev,
    Life,
    Game,
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
    #[arg(long, value_enum, default_value_t = Profile::Default, help = "The profile to use")]
    profile: Profile,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let active_components: Vec<&dyn Component> = match args.profile {
        Profile::Default => vec![&UserComponent, &GitComponent, &SystemComponent],
        Profile::System => vec![&UserComponent, &SystemComponent],
        Profile::Dev => vec![&UserComponent, &GitComponent],
        Profile::Life => vec![&UserComponent],
        Profile::Game => vec![&UserComponent],
    };

    let info = active_components
        .iter()
        .map(|component| component.collect())
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .reduce(|acc, output| acc + output)
        .unwrap_or_else(|| output::ComponentOutput { rows: Vec::new() });

    render(&info);

    Ok(())
}
