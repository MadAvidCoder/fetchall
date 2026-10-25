mod component;
mod components;
mod config;
mod helpers;
mod output;
mod renderer;
mod styled_text;

use crate::component::Component;
use crate::components::{
    dev_env::DevEnvComponent, git::GitComponent, system::SystemComponent, user::UserComponent,
};
use crate::output::ComponentOutput;
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
    #[arg(long, value_name = "NAME", help = "The profile to use")]
    profile: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = config::load_config(None)?;

    let profile_name = args.profile.as_deref().unwrap_or(&config.general.profile);
    let profile = config.profiles.get(profile_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown profile '{}'. Available profiles: {:?}",
            profile_name,
            config.profiles.keys()
        )
    })?;

    let active_components: Vec<&dyn Component> = profile
        .components
        .iter()
        .filter_map(|component_name| match component_name.as_str() {
            "system" => Some(&SystemComponent as &dyn Component),
            "git" => Some(&GitComponent as &dyn Component),
            "dev_env" => Some(&DevEnvComponent as &dyn Component),
            _ => {
                eprintln!("Warning: Unknown component '{}'", component_name);
                None
            }
        })
        .collect();

    let info = active_components
        .iter()
        .filter_map(|component| component.collect().ok())
        .reduce(|acc, output| acc + output)
        .unwrap_or_else(|| {
            UserComponent
                .collect()
                .unwrap_or(ComponentOutput { rows: Vec::new() })
        });

    render(&info);

    Ok(())
}
