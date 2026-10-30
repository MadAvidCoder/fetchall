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
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::{DefaultTerminal, Frame};
use std::fmt;

#[derive(Clone, Copy, ValueEnum, Debug)]
#[repr(usize)]
enum Profile {
    Default,
    System,
    Dev,
    Life,
    Game,
    All,
}
impl TryFrom<usize> for Profile {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Profile::Default),
            1 => Ok(Profile::System),
            2 => Ok(Profile::Dev),
            3 => Ok(Profile::Life),
            4 => Ok(Profile::Game),
            5 => Ok(Profile::All),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Profile::Default => "Default",
            Profile::System => "System",
            Profile::Dev => "Dev",
            Profile::Life => "Life",
            Profile::Game => "Game",
            Profile::All => "All",
        };
        write!(f, "{name}")
    }
}
impl Profile {
    const ALL: [Profile; 6] = [
        Profile::Default,
        Profile::System,
        Profile::Dev,
        Profile::Life,
        Profile::Game,
        Profile::All,
    ];
    fn iter() -> impl Iterator<Item = Profile> {
        Self::ALL.into_iter()
    }
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
    Config {
        #[arg(long, help = "Edit the configuration file")]
        edit: bool,
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

#[derive(Debug, PartialEq, Default)]
enum ConfigScreens {
    #[default]
    Welcome,
    MainMenu {
        selected: usize,
    },
    ProfilesMenu {
        selected: usize,
    },
    ProfileEditor {
        profile: Profile,
    },
    ComponentsMenu,
    AppearanceMenu,
    IntegrationsMenu,
}

#[derive(Default, Debug)]
struct ConfigTui {
    exit: bool,
    screen: ConfigScreens,
}
impl ConfigTui {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_input()?;
        }
        Ok(())
    }
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
    fn handle_input(&mut self) -> Result<()> {
        if let Event::Key(key_event) = event::read()?
            && key_event.kind == KeyEventKind::Press
        {
            self.handle_key_event(key_event);
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if (key_event.code == KeyCode::Char('c')
            && key_event
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL))
            || key_event.code == KeyCode::Esc
            || key_event.code == KeyCode::Char('q')
        {
            self.exit = true;
            return;
        }
        if self.screen == ConfigScreens::Welcome {
            self.screen = ConfigScreens::MainMenu { selected: 0 };
            return;
        }
        match key_event.code {
            KeyCode::Down => {
                if let ConfigScreens::MainMenu { selected } = &mut self.screen {
                    *selected = (*selected + 1) % 5;
                } else if let ConfigScreens::ProfileScreen { selected } = &mut self.screen {
                    *selected = (*selected + 1) % Profile::ALL.len();
                }
            }
            KeyCode::Up => {
                if let ConfigScreens::MainMenu { selected } = &mut self.screen {
                    *selected = (*selected + 4) % 5;
                } else if let ConfigScreens::ProfileScreen { selected } = &mut self.screen {
                    *selected = (*selected + Profile::ALL.len() - 1) % Profile::ALL.len();
                }
            }
            KeyCode::Enter | KeyCode::Space => {
                if let ConfigScreens::MainMenu { selected } = &mut self.screen {
                    match selected {
                        0 => self.screen = ConfigScreens::ProfilesMenu { selected: 0 },
                        1 => todo!("Components"),
                        2 => todo!("Appearance"),
                        3 => todo!("Integeration"),
                        4 => {
                            self.exit = true;
                            return;
                        }
                    };
                } else if let ConfigScreens::ProfilesMenu { selected } = &mut self.screen {
                    self.screen = ConfigScreens::ProfileEditor {
                        profile: Profile::try_from(selected).unwrap_or(Profile::Default),
                    };
                }
            }
            _ => {}
        }
    }

    fn render_welcome(&self, area: Rect, buf: &mut Buffer) {
        let title =
            Line::from("  fetchall | Welcome  ").style(Style::default().fg(Color::Cyan).bold());
        let welcome_text = vec![
            Line::from(""),
            Line::from(""),
            Line::styled(
                "Welcome to Fetchall CLI!",
                Style::default().fg(Color::Yellow).bold(),
            ),
            Line::from(""),
            Line::styled(
                "Please press any key to continue with the setup...",
                Style::default().italic().dim(),
            ),
            Line::from(""),
            Line::from(""),
        ];
        let width = [
            title.width(),
            welcome_text.iter().map(|l| l.width()).max().unwrap_or(0),
        ]
        .into_iter()
        .max()
        .unwrap_or(0)
            + 8;
        let area = area.centered(
            Constraint::Length(width as u16),
            Constraint::Length((welcome_text.len() + 2) as u16),
        );
        Paragraph::new(Text::from(welcome_text).centered())
            .block(
                Block::bordered()
                    .title(title)
                    .title_alignment(Alignment::Center),
            )
            .alignment(Alignment::Center)
            .render(area, buf);
    }
    fn render_profile_editor(&self, area: Rect, buf: &mut Buffer, profile: Profile) {
        todo!();
        let lines = vec![Line::from("")];
    }

    fn render_profiles(&self, area: Rect, buf: &mut Buffer, selected: usize) {
        let mut lines = vec![
            Line::from(""),
            Line::from(""),
            Line::Styled(
                "Select a profile to modify",
                Style::default().fg(Color::White).bold(),
            ),
            Line::from(""),
        ];
        for profile in Profile::iter() {
            lines.push(Line::Styled(
                format!("{}", profile),
                if selected == profile as usize {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ))
        }
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        let title = Line::from("  fetchall | Profiles Config  ")
            .style(Style::default().fg(Color::Cyan).bold());
        let title_bottom = Line::from(
            " Use arrow keys to navigate, Enter to select a profile, and Esc to exit. ".italic(),
        )
        .centered()
        .style(Style::default().fg(Color::DarkGray));

        let line_max_width = lines.iter().map(|f| f.width()).max().unwrap_or(0);
        let width = [line_max_width, title.width(), title_bottom.width()]
            .into_iter()
            .max()
            .unwrap_or(0)
            + 8;
        let area = area.centered(
            Constraint::Length(width as u16),
            Constraint::Length((lines.len() + 2) as u16),
        );

        Paragraph::new(Text::from(lines).centered())
            .block(
                Block::bordered()
                    .title(title)
                    .title_alignment(Alignment::Center)
                    .title_bottom(title_bottom),
            )
            .render(area, buf);
    }

    fn render_main_menu(&self, area: Rect, buf: &mut Buffer, selected: usize) {
        let lines = vec![
            Line::from(""),
            Line::from(""),
            Line::styled(
                "Profile1s",
                if selected == 0 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Line::styled(
                "Components",
                if selected == 1 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Line::styled(
                "Appearance",
                if selected == 2 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Line::styled(
                "Integrations",
                if selected == 3 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Line::from(""),
            Line::styled(
                "Exit",
                if selected == 4 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Line::from(""),
            Line::from(""),
        ];
        let title =
            Line::from("  fetchall | Main Menu  ").style(Style::default().fg(Color::Cyan).bold());
        let title_bottom =
            Line::from(" Use arrow keys to navigate, Enter to select, and Esc to exit. ".italic())
                .centered()
                .style(Style::default().fg(Color::DarkGray));

        let line_max_width = lines.iter().map(|f| f.width()).max().unwrap_or(0);
        let width = [line_max_width, title.width(), title_bottom.width()]
            .into_iter()
            .max()
            .unwrap_or(0)
            + 8;
        let area = area.centered(
            Constraint::Length(width as u16),
            Constraint::Length((lines.len() + 2) as u16),
        );

        Paragraph::new(Text::from(lines).centered())
            .block(
                Block::bordered()
                    .title(title)
                    .title_alignment(Alignment::Center)
                    .title_bottom(title_bottom),
            )
            .render(area, buf);
    }
}

impl Widget for &ConfigTui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.screen {
            ConfigScreens::Welcome => self.render_welcome(area, buf),
            ConfigScreens::MainMenu { selected } => self.render_main_menu(area, buf, selected),
            ConfigScreens::ProfilesMenu { selected } => self.render_profiles(area, buf, selected),
            ConfigScreens::ProfileEditor => todo!(),
            ConfigScreens::ComponentsMenu => todo!(),
            ConfigScreens::AppearanceMenu => todo!(),
            ConfigScreens::IntegrationsMenu => todo!(),
        };
    }
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
                            println!("GitHub is already authenticated and the app is installed.");
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
            Commands::Config { edit: _ } => {
                return ratatui::run(|terminal| ConfigTui::default().run(terminal));
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
