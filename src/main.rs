mod component;
mod components;
mod config;
mod github_auth;
mod hackatime_auth;
mod helpers;
mod logo;
mod output;
mod renderer;
mod styled_text;

use crate::component::{Component, get_component_by_name};
use crate::components::user::UserComponent;
use crate::output::ComponentOutput;
use crate::renderer::render;
use crate::styled_text::StyledText;
use anyhow::Result;
use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, ValueEnum, Debug)]
enum Profile {
    Default,
    System,
    Dev,
    Life,
    Game,
    All,
}

#[derive(Debug, clap::Subcommand)]
#[group(multiple = false)]
enum Commands {
    Auth {
        #[arg(value_enum, help = "The API to authenticate")]
        service: AuthService,
        #[arg(
            long,
            group = "auth_action",
            help = "Authenticate fetchall to use the API"
        )]
        login: bool,
        #[arg(long, group = "auth_action", help = "Check authentication status")]
        status: bool,
        #[arg(long, group = "auth_action", help = "Revoke authentication")]
        revoke: bool,
    },
}

#[derive(Debug, clap::ValueEnum, Clone, Copy)]
enum AuthService {
    Github,
    Hackatime,
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
    #[command(subcommand, help = "Manage API authentication")]
    command: Option<Commands>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(command) = args.command {
        match command {
            Commands::Auth {
                service: AuthService::Github,
                login,
                status,
                revoke,
            } => {
                if login || !status && !revoke {
                    github_auth::login()?;
                } else if status {
                    match github_auth::check_login().unwrap_or((false, false)) {
                        (true, true) => {
                            println!("GitHub is authenticated and the app is installed.");
                        }
                        (true, false) => {
                            println!("GitHub is authenticated, but the app is not installed.");
                            println!();
                            print!("Please go to ");
                            print!(
                                "{}",
                                StyledText::new(github_auth::CONTINUE_URL)
                                    .bright_yellow()
                                    .italic()
                                    .render()
                            );
                            println!(" and install Fetchall CLI Github to complete setup.");
                        }
                        (false, _) => {
                            print!("You have not authenticated GitHub. Please run `");
                            print!(
                                "{}",
                                StyledText::new("fetchall auth github")
                                    .bright_yellow()
                                    .bold()
                                    .render()
                            );
                            println!("` to authenticate.");
                        }
                    }
                } else if revoke {
                    github_auth::revoke()?;
                }
                return Ok(());
            }
            Commands::Auth {
                service: AuthService::Hackatime,
                login,
                status,
                revoke,
            } => {
                if login || !status && !revoke {
                    hackatime_auth::login()?;
                } else if status {
                    if hackatime_auth::check_login()? {
                        println!("Hackatime is authenticated.");
                    } else {
                        print!("You have not authenticated hackatime. Please run `");
                        print!(
                            "{}",
                            StyledText::new("fetchall auth hackatime")
                                .bright_yellow()
                                .bold()
                                .render()
                        );
                        println!("` to authenticate.");
                    }
                } else if revoke {
                    hackatime_auth::revoke()?;
                }
                return Ok(());
            }
        }
    }

    let config = config::load_config(None)?;

    let profile_name = args.profile.as_deref().unwrap_or(&config.general.profile);
    let profile = config.profiles.get(profile_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown profile '{}'. Available profiles: {:?}",
            profile_name,
            config.profiles.keys()
        )
    })?;

    let active_components: Vec<Box<dyn Component>> = profile
        .components
        .iter()
        .filter_map(|component_name| get_component_by_name(component_name, &config))
        .collect();

    let info = active_components
        .iter()
        .filter_map(|component| component.collect().ok())
        .reduce(|acc, output| acc + output)
        .unwrap_or_else(|| {
            UserComponent::new(&config::UserConfig::default(), "auto", &true)
                .collect()
                .unwrap_or(ComponentOutput { rows: Vec::new() })
        });

    let logo = logo::get_logo(&config.appearance.logo, &config.appearance.logo_colour);
    render(
        &logo,
        &info,
        config.appearance.logo_gap,
        config.appearance.label_gap,
    );

    Ok(())
}
