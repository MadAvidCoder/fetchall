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

use std::sync::mpsc::TryRecvError;
use std::thread;

use crate::component::{COMPONENTS, Component, get_component_by_name};
use crate::components::user::UserComponent;
use crate::output::ComponentOutput;
use crate::renderer::render;
use crate::styled_text::StyledText;
use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph, Widget, Wrap};
use ratatui::{DefaultTerminal, Frame};

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
        profile: String,
        selected: usize,
    },
    EnableComponentsMenu {
        profile: String,
        selected: usize,
        moving: Option<usize>,
    },
    NewProfileMenu {
        name: String,
        cursor: usize,
    },
    RenameProfileMenu {
        profile: String,
        name: String,
        cursor: usize,
    },
    FileInputMenu {
        title: String,
        input: String,
        cursor: usize,
    },
    AppearanceMenu {
        selected: usize,
    },
    LogoMenu {
        selected: usize,
    },
    OsLogoMenu {
        selected: usize,
    },
    ColourMenu {
        selected: usize,
    },
    LogoColourMenu {
        selected: usize,
    },
    MainColourMenu {
        selected: usize,
    },
    LayoutMenu {
        selected: usize,
    },
    HackatimeLoginScreen {
        done1: bool,
        done2: bool,
        err: Option<String>,
    },
    GitHubLoginScreen {
        done1: bool,
        done2: bool,
        err: Option<String>,
        url: Option<String>,
        code: Option<String>,
    },
    ComponentsMenu,
    IntegrationsMenu {
        selected: usize,
        gh_status: (bool, bool),
        hac_status: bool,
    },
}

#[derive(Default, Debug)]
struct ConfigTui {
    exit: bool,
    screen: ConfigScreens,
    rx: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
    ghrx: Option<std::sync::mpsc::Receiver<Result<(String, String), String>>>,
}
impl ConfigTui {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            if self.rx.is_some() {
                self.poll_hackatime();
            }
            if self.ghrx.is_some() {
                self.poll_github();
            }
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
            || ((key_event.code == KeyCode::Esc || key_event.code == KeyCode::Char('q'))
                && !matches!(self.screen, ConfigScreens::NewProfileMenu { .. })
                && !matches!(self.screen, ConfigScreens::RenameProfileMenu { .. })
                && !matches!(self.screen, ConfigScreens::FileInputMenu { .. })
                && !matches!(self.screen, ConfigScreens::HackatimeLoginScreen { .. })
                && !matches!(self.screen, ConfigScreens::GitHubLoginScreen { .. }))
        {
            self.exit = true;
            return;
        }
        if self.screen == ConfigScreens::Welcome {
            self.screen = ConfigScreens::MainMenu { selected: 0 };
            return;
        }
        if let ConfigScreens::NewProfileMenu { name, cursor } = &mut self.screen {
            match key_event.code {
                KeyCode::Esc => {
                    self.screen = ConfigScreens::ProfilesMenu { selected: 0 };
                }
                KeyCode::Char(c) => {
                    name.insert(*cursor, c);
                    *cursor += 1;
                }
                KeyCode::Backspace if *cursor > 0 => {
                    name.remove(*cursor - 1);
                    *cursor -= 1;
                }
                KeyCode::Delete if *cursor < name.len() => {
                    name.remove(*cursor);
                }
                KeyCode::Left if *cursor > 0 => *cursor -= 1,
                KeyCode::Right if *cursor < name.len() => *cursor += 1,
                KeyCode::Enter if !name.trim().is_empty() => {
                    config::add_profile(&name.trim().to_lowercase()).unwrap();

                    self.screen = ConfigScreens::EnableComponentsMenu {
                        profile: name.trim().to_lowercase(),
                        selected: 0,
                        moving: None,
                    };
                }
                _ => {}
            }
            return;
        }
        if let ConfigScreens::RenameProfileMenu {
            name,
            cursor,
            profile,
        } = &mut self.screen
        {
            match key_event.code {
                KeyCode::Esc => {
                    self.screen = ConfigScreens::ProfileEditor {
                        profile: profile.clone(),
                        selected: 0,
                    };
                }
                KeyCode::Char(c)
                    if c != ' ' && c != '\t' && c != '\n' && c != '\r' && c != '\0' =>
                {
                    name.insert(*cursor, c);
                    *cursor += 1;
                }
                KeyCode::Backspace if *cursor > 0 => {
                    name.remove(*cursor - 1);
                    *cursor -= 1;
                }
                KeyCode::Delete if *cursor < name.len() => {
                    name.remove(*cursor);
                }
                KeyCode::Left if *cursor > 0 => *cursor -= 1,
                KeyCode::Right if *cursor < name.len() => *cursor += 1,
                KeyCode::Enter if !name.trim().is_empty() => {
                    config::rename_profile(profile, &name.trim().to_lowercase()).unwrap();

                    self.screen = ConfigScreens::ProfileEditor {
                        profile: name.trim().to_lowercase(),
                        selected: 0,
                    }
                }
                _ => {}
            }
            return;
        }
        if let ConfigScreens::FileInputMenu {
            input,
            cursor,
            title,
        } = &mut self.screen
        {
            match key_event.code {
                KeyCode::Esc => self.screen = ConfigScreens::LogoMenu { selected: 3 },
                KeyCode::Char(c) if c != '\t' && c != '\n' && c != '\r' && c != '\0' => {
                    input.insert(*cursor, c);
                    *cursor += 1;
                }
                KeyCode::Backspace if *cursor > 0 => {
                    input.remove(*cursor - 1);
                    *cursor -= 1;
                }
                KeyCode::Delete if *cursor < input.len() => {
                    input.remove(*cursor);
                }
                KeyCode::Left if *cursor > 0 => *cursor -= 1,
                KeyCode::Right if *cursor < input.len() => *cursor += 1,
                KeyCode::Enter
                    if match std::fs::read_to_string(&input) {
                        Ok(c) if !c.trim().is_empty() => {
                            config::set_logo(input).unwrap();
                            true
                        }
                        _ => {
                            *title = "Invalid file path or empty file\nPlease check the absolute path to an ASCII art file:".to_string();
                            false
                        }
                    } =>
                {
                    self.screen = ConfigScreens::LogoMenu { selected: 3 };
                }
                _ => {}
            }
            return;
        }
        if let ConfigScreens::HackatimeLoginScreen { done1, done2, .. } = &mut self.screen {
            match key_event.code {
                KeyCode::Enter | KeyCode::Char(' ') if *done1 && *done2 => {
                    self.screen = ConfigScreens::IntegrationsMenu {
                        selected: 1,
                        gh_status: github_auth::check_login().unwrap_or((false, false)),
                        hac_status: hackatime_auth::check_login().unwrap_or(false),
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.screen = ConfigScreens::IntegrationsMenu {
                        selected: 1,
                        gh_status: github_auth::check_login().unwrap_or((false, false)),
                        hac_status: hackatime_auth::check_login().unwrap_or(false),
                    }
                }
                _ => {}
            }
            return;
        }
        if let ConfigScreens::GitHubLoginScreen { done1, done2, .. } = &mut self.screen {
            match key_event.code {
                KeyCode::Enter | KeyCode::Char(' ') if *done1 && *done2 => {
                    self.screen = ConfigScreens::IntegrationsMenu {
                        selected: 1,
                        gh_status: github_auth::check_login().unwrap_or((false, false)),
                        hac_status: hackatime_auth::check_login().unwrap_or(false),
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.screen = ConfigScreens::IntegrationsMenu {
                        selected: 1,
                        gh_status: github_auth::check_login().unwrap_or((false, false)),
                        hac_status: hackatime_auth::check_login().unwrap_or(false),
                    }
                }
                _ => {}
            }
            return;
        }

        match key_event.code {
            KeyCode::Down => {
                if let ConfigScreens::MainMenu { selected } = &mut self.screen {
                    *selected = (*selected + 1) % 4;
                } else if let ConfigScreens::ProfilesMenu { selected } = &mut self.screen {
                    *selected = (*selected + 1) % (config::get_profiles().len() + 3);
                } else if let ConfigScreens::ProfileEditor { selected, .. } = &mut self.screen {
                    *selected = (*selected + 1) % 4;
                } else if let ConfigScreens::EnableComponentsMenu {
                    selected,
                    moving,
                    profile,
                } = &mut self.screen
                {
                    if let Some(moving) = moving {
                        let config = config::load_config(None).unwrap();
                        let enabled = &config.profiles[profile].components;
                        if *moving + 1 < enabled.len() {
                            config::move_component_prof(profile, &enabled[*moving], *moving + 1)
                                .unwrap();
                            *moving += 1;
                            *selected = *moving
                        }
                    } else {
                        *selected = (*selected + 1) % (component::COMPONENTS.len() + 1);
                    }
                } else if let ConfigScreens::AppearanceMenu { selected } = &mut self.screen {
                    *selected = (*selected + 1) % 4;
                } else if let ConfigScreens::LogoMenu { selected } = &mut self.screen {
                    *selected = (*selected + 1) % 6;
                } else if let ConfigScreens::OsLogoMenu { selected } = &mut self.screen {
                    *selected = (*selected + 1) % (logo::OS_LOGOS.len() + 1);
                } else if let ConfigScreens::ColourMenu { selected } = &mut self.screen {
                    *selected = (*selected + 1) % 4;
                } else if let ConfigScreens::LogoColourMenu { selected } = &mut self.screen {
                    *selected = (*selected + 1) % (styled_text::COLOURS.len() + 2);
                } else if let ConfigScreens::MainColourMenu { selected } = &mut self.screen {
                    *selected = (*selected + 1) % (styled_text::COLOURS.len() + 2);
                } else if let ConfigScreens::LayoutMenu { selected } = &mut self.screen {
                    *selected = (*selected + 1) % 5;
                } else if let ConfigScreens::IntegrationsMenu { selected, .. } = &mut self.screen {
                    *selected = (*selected + 1) % 3;
                }
            }
            KeyCode::Up => {
                if let ConfigScreens::MainMenu { selected } = &mut self.screen {
                    *selected = (*selected + 4 - 1) % 4;
                } else if let ConfigScreens::ProfilesMenu { selected } = &mut self.screen {
                    *selected = (*selected + config::get_profiles().len() + 3 - 1)
                        % (config::get_profiles().len() + 3);
                } else if let ConfigScreens::ProfileEditor { selected, .. } = &mut self.screen {
                    *selected = (*selected + 4 - 1) % 4;
                } else if let ConfigScreens::EnableComponentsMenu {
                    selected,
                    moving,
                    profile,
                } = &mut self.screen
                {
                    if let Some(moving) = moving {
                        let config = config::load_config(None).unwrap();
                        let enabled = &config.profiles[profile].components;
                        if *moving > 0 {
                            config::move_component_prof(profile, &enabled[*moving], *moving - 1)
                                .unwrap();
                            *moving -= 1;
                            *selected = *moving
                        }
                    } else {
                        *selected = (*selected + component::COMPONENTS.len() + 1 - 1)
                            % (component::COMPONENTS.len() + 1);
                    }
                } else if let ConfigScreens::AppearanceMenu { selected } = &mut self.screen {
                    *selected = (*selected + 4 - 1) % 4;
                } else if let ConfigScreens::LogoMenu { selected } = &mut self.screen {
                    *selected = (*selected + 6 - 1) % 6;
                } else if let ConfigScreens::OsLogoMenu { selected } = &mut self.screen {
                    *selected =
                        (*selected + logo::OS_LOGOS.len() + 1 - 1) % (logo::OS_LOGOS.len() + 1);
                } else if let ConfigScreens::ColourMenu { selected } = &mut self.screen {
                    *selected = (*selected + 4 - 1) % 4;
                } else if let ConfigScreens::LogoColourMenu { selected } = &mut self.screen {
                    *selected = (*selected + styled_text::COLOURS.len() + 2 - 1)
                        % (styled_text::COLOURS.len() + 2);
                } else if let ConfigScreens::MainColourMenu { selected } = &mut self.screen {
                    *selected = (*selected + styled_text::COLOURS.len() + 2 - 1)
                        % (styled_text::COLOURS.len() + 2);
                } else if let ConfigScreens::LayoutMenu { selected } = &mut self.screen {
                    *selected = (*selected + 5 - 1) % 5;
                } else if let ConfigScreens::IntegrationsMenu { selected, .. } = &mut self.screen {
                    *selected = (*selected + 3 - 1) % 3;
                }
            }
            KeyCode::Left if let ConfigScreens::LayoutMenu { selected } = &mut self.screen => {
                if *selected == 1 {
                    config::set_logo_gap(
                        config::load_config(None)
                            .unwrap()
                            .appearance
                            .logo_gap
                            .saturating_sub(1),
                    )
                    .unwrap()
                } else if *selected == 2 {
                    config::set_label_gap(
                        config::load_config(None)
                            .unwrap()
                            .appearance
                            .label_gap
                            .saturating_sub(1),
                    )
                    .unwrap()
                }
            }
            KeyCode::Right if let ConfigScreens::LayoutMenu { selected } = &mut self.screen => {
                if *selected == 1 {
                    config::set_logo_gap(
                        config::load_config(None)
                            .unwrap()
                            .appearance
                            .logo_gap
                            .saturating_add(1),
                    )
                    .unwrap()
                } else if *selected == 2 {
                    config::set_label_gap(
                        config::load_config(None)
                            .unwrap()
                            .appearance
                            .label_gap
                            .saturating_add(1),
                    )
                    .unwrap()
                }
            }
            KeyCode::Char('m') => {
                if let ConfigScreens::EnableComponentsMenu {
                    selected,
                    moving,
                    profile,
                } = &mut self.screen
                {
                    if moving.is_none()
                        && *selected < component::COMPONENTS.len()
                        && config::load_config(None)
                            .unwrap()
                            .profiles
                            .get(&profile.to_string().to_lowercase())
                            .map(|f| f.components.clone())
                            .unwrap_or_default()
                            .contains(
                                &Self::get_ordered_components(profile)[*selected]
                                    .0
                                    .to_string(),
                            )
                    {
                        *moving = Some(*selected);
                    } else {
                        *moving = None;
                    }
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let ConfigScreens::MainMenu { selected } = &mut self.screen {
                    match selected {
                        0 => self.screen = ConfigScreens::ProfilesMenu { selected: 0 },
                        // 1 => self.screen = ConfigScreens::ComponentsMenu,
                        1 => self.screen = ConfigScreens::AppearanceMenu { selected: 0 },
                        2 => {
                            let hackatime = hackatime_auth::check_login().unwrap_or(false);
                            let gh = github_auth::check_login().unwrap_or((false, false));
                            self.screen = ConfigScreens::IntegrationsMenu {
                                selected: 0,
                                gh_status: gh,
                                hac_status: hackatime,
                            }
                        }
                        3 => self.exit = true,
                        _ => {}
                    };
                } else if let ConfigScreens::ProfilesMenu { selected } = &mut self.screen {
                    if *selected == config::get_profiles().len() + 2 {
                        self.screen = ConfigScreens::MainMenu { selected: 0 };
                    } else if *selected == config::get_profiles().len() + 1 {
                        config::reset_profiles().unwrap();
                        self.screen = ConfigScreens::ProfilesMenu { selected: 0 };
                    } else if *selected == config::get_profiles().len() {
                        self.screen = ConfigScreens::NewProfileMenu {
                            name: String::new(),
                            cursor: 0,
                        }
                    } else {
                        self.screen = ConfigScreens::ProfileEditor {
                            selected: 0,
                            profile: config::get_profiles()[*selected].to_string(),
                        };
                    }
                } else if let ConfigScreens::ProfileEditor { profile, selected } = &mut self.screen
                {
                    match selected {
                        0 => {
                            self.screen = ConfigScreens::EnableComponentsMenu {
                                profile: profile.clone(),
                                selected: 0,
                                moving: None,
                            }
                        }
                        1 => {
                            self.screen = ConfigScreens::RenameProfileMenu {
                                profile: profile.clone(),
                                name: profile.clone(),
                                cursor: profile.len(),
                            }
                        }
                        2 => {
                            let profile_name = profile.to_string().to_lowercase();
                            config::remove_profile(&profile_name).unwrap();
                            self.screen = ConfigScreens::ProfilesMenu { selected: 0 };
                        }
                        3 => {
                            self.screen = ConfigScreens::ProfilesMenu {
                                selected: config::get_profiles()
                                    .iter()
                                    .position(|p| p == profile)
                                    .unwrap_or(0),
                            }
                        }
                        _ => {}
                    }
                } else if let ConfigScreens::EnableComponentsMenu {
                    profile,
                    selected,
                    moving,
                } = &mut self.screen
                {
                    if moving.is_some() {
                        *moving = None;
                    } else if *selected == component::COMPONENTS.len() {
                        self.screen = ConfigScreens::ProfileEditor {
                            selected: 0,
                            profile: profile.clone(),
                        };
                    } else {
                        let sel_comp = Self::get_ordered_components(profile)[*selected].0;
                        let config = config::load_config(None).unwrap();
                        if config
                            .profiles
                            .get(&profile.to_string().to_lowercase())
                            .map(|a| a.components.clone())
                            .unwrap_or_default()
                            .contains(&sel_comp.to_string())
                        {
                            config::remove_component_from_profile(
                                &profile.to_string().to_lowercase(),
                                sel_comp,
                            )
                            .unwrap();
                        } else {
                            config::add_component_to_profile(
                                &profile.to_string().to_lowercase(),
                                sel_comp,
                            )
                            .unwrap();
                        }

                        *selected = Self::get_ordered_components(profile)
                            .iter()
                            .position(|(c, _)| c == &sel_comp)
                            .unwrap_or(0);
                    }
                } else if let ConfigScreens::AppearanceMenu { selected } = &mut self.screen {
                    match selected {
                        0 => self.screen = ConfigScreens::LogoMenu { selected: 0 },
                        1 => self.screen = ConfigScreens::ColourMenu { selected: 0 },
                        2 => self.screen = ConfigScreens::LayoutMenu { selected: 0 },
                        3 => self.screen = ConfigScreens::MainMenu { selected: 2 },
                        _ => {}
                    }
                } else if let ConfigScreens::LogoMenu { selected } = &mut self.screen {
                    match selected {
                        0 => config::set_logo("auto").unwrap(),
                        1 => self.screen = ConfigScreens::OsLogoMenu { selected: 0 },
                        2 => config::set_logo("fetchall").unwrap(),
                        3 => {
                            let cur = config::load_config(None).unwrap().appearance.logo.clone();
                            self.screen = ConfigScreens::FileInputMenu {
                                input: if cur != "none"
                                    && cur != "auto"
                                    && cur != "fetchall"
                                    && !logo::OS_LOGOS.contains(&cur.as_str())
                                {
                                    cur
                                } else {
                                    String::new()
                                },
                                cursor: 0,
                                title: "Please enter the absolute path to the ASCII art file:"
                                    .to_string(),
                            }
                        }
                        4 => config::set_logo("none").unwrap(),
                        5 => self.screen = ConfigScreens::AppearanceMenu { selected: 0 },
                        _ => {}
                    }
                } else if let ConfigScreens::OsLogoMenu { selected } = &mut self.screen {
                    if *selected < logo::OS_LOGOS.len() {
                        config::set_logo(logo::OS_LOGOS[*selected]).unwrap();
                    }
                    self.screen = ConfigScreens::LogoMenu { selected: 1 };
                } else if let ConfigScreens::ColourMenu { selected } = &mut self.screen {
                    match selected {
                        1 => self.screen = ConfigScreens::LogoColourMenu { selected: 0 },
                        0 => self.screen = ConfigScreens::MainColourMenu { selected: 0 },
                        2 => {
                            config::set_colour("auto").unwrap();
                            config::set_logo_colour("auto").unwrap();
                            self.screen = ConfigScreens::ColourMenu { selected: 0 };
                        }
                        3 => self.screen = ConfigScreens::AppearanceMenu { selected: 1 },
                        _ => {}
                    }
                } else if let ConfigScreens::LogoColourMenu { selected } = &mut self.screen {
                    if *selected == 0 {
                        config::set_logo_colour("auto").unwrap();
                    } else if *selected <= styled_text::COLOURS.len() {
                        config::set_logo_colour(styled_text::COLOURS[*selected - 1]).unwrap();
                    }
                    self.screen = ConfigScreens::ColourMenu { selected: 1 };
                } else if let ConfigScreens::MainColourMenu { selected } = &mut self.screen {
                    if *selected == 0 {
                        config::set_colour("auto").unwrap();
                    } else if *selected <= styled_text::COLOURS.len() {
                        config::set_colour(styled_text::COLOURS[*selected - 1]).unwrap();
                    }
                    self.screen = ConfigScreens::ColourMenu { selected: 0 };
                } else if let ConfigScreens::LayoutMenu { selected } = &mut self.screen {
                    if *selected == 0 {
                        config::set_title_visibility(
                            !config::load_config(None)
                                .unwrap()
                                .appearance
                                .show_section_titles,
                        )
                        .unwrap();
                    } else if *selected == 3 {
                        config::set_section_gap(
                            !config::load_config(None)
                                .unwrap()
                                .appearance
                                .show_section_gap,
                        )
                        .unwrap();
                    } else if *selected == 4 {
                        self.screen = ConfigScreens::AppearanceMenu { selected: 2 };
                    }
                } else {
                    match if let ConfigScreens::IntegrationsMenu {
                        selected,
                        gh_status,
                        hac_status,
                    } = &mut self.screen
                    {
                        match selected {
                            0 if !gh_status.0 => 1,
                            0 if !gh_status.1 => 2,
                            1 if !*hac_status => 3,
                            2 => {
                                self.screen = ConfigScreens::MainMenu { selected: 3 };
                                0
                            }
                            _ => 0,
                        }
                    } else {
                        0
                    } {
                        1 => {
                            let (tx, rx) = std::sync::mpsc::channel();
                            thread::spawn(move || {
                                let device = match github_auth::silent_login() {
                                    Ok(d) => {
                                        let _ = tx.send(Ok((
                                            d.verification_uri.clone(),
                                            d.user_code.clone(),
                                        )));
                                        d
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Err(e.to_string()));
                                        return;
                                    }
                                };
                                match github_auth::silent_login_2(device) {
                                    Ok(_) => {
                                        let _ = tx.send(Ok((String::new(), String::new())));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Err(e.to_string()));
                                    }
                                };
                            });
                            self.screen = ConfigScreens::GitHubLoginScreen {
                                done1: false,
                                done2: false,
                                url: None,
                                code: None,
                                err: None,
                            };
                            self.ghrx = Some(rx);
                        }
                        2 => {
                            self.screen = ConfigScreens::GitHubLoginScreen {
                                done1: true,
                                done2: true,
                                url: None,
                                code: None,
                                err: None,
                            };
                            open::that(github_auth::CONTINUE_URL).unwrap_or(());
                        }
                        3 => {
                            let (tx, rx) = std::sync::mpsc::channel();
                            thread::spawn(move || {
                                let code = match hackatime_auth::login_silent() {
                                    Ok(c) => {
                                        let _ = tx.send(Ok(()));
                                        c
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Err(e.to_string()));
                                        return;
                                    }
                                };
                                match hackatime_auth::login_silent_2(code) {
                                    Ok(_) => {
                                        let _ = tx.send(Ok(()));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Err(e.to_string()));
                                    }
                                };
                            });

                            self.screen = ConfigScreens::HackatimeLoginScreen {
                                done1: false,
                                done2: false,
                                err: None,
                            };
                            self.rx = Some(rx);
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Char('d')
                if let ConfigScreens::IntegrationsMenu {
                    selected,
                    gh_status,
                    hac_status,
                } = &mut self.screen =>
            {
                match selected {
                    0 if gh_status.0 => {
                        github_auth::revoke(true).unwrap();
                        *gh_status = github_auth::check_login().unwrap_or((false, false));
                    }
                    1 if *hac_status => {
                        hackatime_auth::revoke(true).unwrap();
                        *hac_status = hackatime_auth::check_login().unwrap_or(false);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    fn poll_hackatime(&mut self) {
        let Some(rx) = &self.rx else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(())) => {
                if let ConfigScreens::HackatimeLoginScreen { done1, done2, .. } = &mut self.screen {
                    if !*done1 {
                        *done1 = true;
                    } else if !*done2 {
                        *done2 = true;
                    }
                }
            }
            Ok(Err(e)) => {
                if let ConfigScreens::HackatimeLoginScreen { err, .. } = &mut self.screen {
                    *err = Some(e);
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.rx = None;
            }
        }
    }
    fn poll_github(&mut self) {
        let Some(rx) = &self.ghrx else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok((url, code))) => {
                if let ConfigScreens::GitHubLoginScreen {
                    done1,
                    done2,
                    url: log_url,
                    code: log_code,
                    ..
                } = &mut self.screen
                {
                    if !*done1 {
                        *done1 = true;
                        *log_url = Some(url);
                        *log_code = Some(code);
                    } else if !*done2 {
                        *done2 = true;
                    }
                }
            }
            Ok(Err(e)) => {
                if let ConfigScreens::GitHubLoginScreen { err, .. } = &mut self.screen {
                    *err = Some(e);
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.ghrx = None;
            }
        }
    }

    fn render_hackatime_login(
        &self,
        area: Rect,
        buf: &mut Buffer,
        done1: bool,
        done2: bool,
        err: &Option<String>,
    ) {
        let title = Line::from("  fetchall | Hackatime Login  ")
            .style(Style::default().fg(Color::Cyan).bold());
        let title_bottom = Line::from(" Press Esc to cancel authorisation and return to the menu ")
            .centered()
            .style(Style::default().fg(Color::DarkGray));

        let url = format!(
            "https://hackatime.hackclub.com/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope=profile+read",
            hackatime_auth::CLIENT_ID,
            hackatime_auth::REDIRECT_URI
        );
        let lines = vec![
            Line::from(""),
            Line::from(""),
            Line::from("Please go to the following URL and authorize fetchall-cli:"),
            Line::from(""),
            Line::styled(&url, Style::default().fg(Color::Yellow).italic()),
            Line::from(""),
            Line::from(if let Some(e) = err {
                format!("Error: {}", e).red().bold()
            } else if !done1 {
                "Waiting for Hackatime callback...".dark_gray().italic()
            } else if !done2 {
                "Waiting for token exchange...".dark_gray().italic()
            } else {
                "Authorization complete! Press Enter to return."
                    .green()
                    .bold()
            }),
            Line::from(""),
            Line::from(""),
            Line::from(""),
        ];
        let line_max_width = lines.iter().map(|f| f.width()).max().unwrap_or(0);
        let width = ([line_max_width, title.width(), title_bottom.width()]
            .into_iter()
            .max()
            .unwrap_or(0)
            + 8)
        .min((area.width as usize).saturating_sub(2));
        let area = area.centered(
            Constraint::Length(width as u16),
            Constraint::Length(((lines.len() + 2) as u16).min(area.height.saturating_sub(2))),
        );

        Paragraph::new(Text::from(lines).centered())
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .title(title)
                    .title_alignment(Alignment::Center)
                    .title_bottom(title_bottom),
            )
            .render(area, buf);
    }

    fn render_github_login(
        &self,
        area: Rect,
        buf: &mut Buffer,
        done1: bool,
        done2: bool,
        code: &Option<String>,
        url: &Option<String>,
        err: &Option<String>,
    ) {
        let title = Line::from("  fetchall | GitHub Login  ")
            .style(Style::default().fg(Color::Cyan).bold());
        let title_bottom = Line::from(" Press Esc to cancel authorisation and return to the menu ")
            .centered()
            .style(Style::default().fg(Color::DarkGray));

        let lines = if let Some(e) = err {
            vec![
                Line::from(""),
                Line::from(""),
                Line::from(format!("Error: {}", e).red().bold()),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(""),
            ]
        } else if !done1 {
            vec![
                Line::from(""),
                Line::from(""),
                Line::from("Obtaining device code. Please wait..."),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(""),
            ]
        } else if !done2 && let Some(url) = url {
            vec![
                Line::from(""),
                Line::from(""),
                Line::from("Please go to the following URL and authorize fetchall-cli:"),
                Line::styled(url, Style::default().fg(Color::Yellow).italic()),
                Line::from(""),
                Line::from(vec![
                    "Enter the following code when prompted ".bold(),
                    "(copied to clipboard)".italic().dark_gray(),
                ]),
                Line::from(
                    code.as_ref()
                        .unwrap_or(&String::new())
                        .to_string()
                        .bold()
                        .yellow(),
                ),
                Line::from(""),
                Line::from(""),
                Line::from("Waiting for token exchange...".dark_gray().italic()),
                Line::from(""),
            ]
        } else {
            vec![
                Line::from(""),
                Line::from(""),
                Line::from(
                    "CLI token succesfully obtained and saved to OS store."
                        .green()
                        .bold(),
                ),
                Line::from(""),
                Line::from(""),
                Line::from("To complete setup, install fetchall-cli to your GitHub Account:"),
                Line::styled(
                    github_auth::CONTINUE_URL,
                    Style::default().fg(Color::Yellow).italic(),
                ),
                Line::from(""),
                Line::from(""),
                Line::from(
                    "Once you are done, press Enter to return"
                        .dark_gray()
                        .italic(),
                ),
                Line::from(""),
            ]
        };
        let line_max_width = lines.iter().map(|f| f.width()).max().unwrap_or(0);
        let width = ([line_max_width, title.width(), title_bottom.width()]
            .into_iter()
            .max()
            .unwrap_or(0)
            + 8)
        .min((area.width as usize).saturating_sub(2));
        let area = area.centered(
            Constraint::Length(width as u16),
            Constraint::Length(((lines.len() + 2) as u16).min(area.height.saturating_sub(2))),
        );

        Paragraph::new(Text::from(lines).centered())
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .title(title)
                    .title_alignment(Alignment::Center)
                    .title_bottom(title_bottom),
            )
            .render(area, buf);
    }

    fn render_logo_colour(&self, area: Rect, buf: &mut Buffer, selected: usize) {
        let cur = config::load_config(None)
            .unwrap()
            .appearance
            .logo_colour
            .clone();
        let mut lines = vec![
            Line::from(""),
            Line::from(""),
            Line::styled(
                "Automatic",
                if selected == 0 {
                    if cur == "auto" {
                        Style::default().fg(Color::LightGreen).bold().italic()
                    } else {
                        Style::default().fg(Color::Yellow).bold()
                    }
                } else {
                    if cur == "auto" {
                        Style::default().fg(Color::Green).italic()
                    } else {
                        Style::default().fg(Color::White)
                    }
                },
            ),
            Line::from(""),
        ];
        for col in styled_text::COLOURS.iter().enumerate() {
            lines.push(Line::styled(
                *col.1,
                if selected == col.0 + 1 {
                    if cur == *col.1 {
                        Style::default().fg(Color::LightGreen).bold().italic()
                    } else {
                        Style::default().fg(Color::Yellow).bold()
                    }
                } else {
                    if cur == *col.1 {
                        Style::default().fg(Color::Green).italic()
                    } else {
                        Style::default().fg(Color::White)
                    }
                },
            ));
        }
        lines.push(Line::from(""));
        lines.push(Line::styled(
            "Back",
            if selected == styled_text::COLOURS.len() + 1 {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        let title =
            Line::from("  fetchall | Logo Colour  ").style(Style::default().fg(Color::Cyan).bold());
        let title_bottom =
            Line::from(" Use the arrow keys to navigate, Enter to select, and Esc to cancel ")
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

    fn render_colour_choice_menu(&self, area: Rect, buf: &mut Buffer, selected: usize) {
        let cur = config::load_config(None).unwrap().appearance.colour.clone();
        let mut lines = vec![
            Line::from(""),
            Line::from(""),
            Line::styled(
                "Automatic",
                if selected == 0 {
                    if cur == "auto" {
                        Style::default().fg(Color::LightGreen).bold().italic()
                    } else {
                        Style::default().fg(Color::Yellow).bold()
                    }
                } else {
                    if cur == "auto" {
                        Style::default().fg(Color::Green).italic()
                    } else {
                        Style::default().fg(Color::White)
                    }
                },
            ),
            Line::from(""),
        ];
        for col in styled_text::COLOURS.iter().enumerate() {
            lines.push(Line::styled(
                *col.1,
                if selected == col.0 + 1 {
                    if cur == *col.1 {
                        Style::default().fg(Color::LightGreen).bold().italic()
                    } else {
                        Style::default().fg(Color::Yellow).bold()
                    }
                } else {
                    if cur == *col.1 {
                        Style::default().fg(Color::Green).italic()
                    } else {
                        Style::default().fg(Color::White)
                    }
                },
            ));
        }
        lines.push(Line::from(""));
        lines.push(Line::styled(
            "Back",
            if selected == styled_text::COLOURS.len() + 1 {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        let title =
            Line::from("  fetchall | Text Colour  ").style(Style::default().fg(Color::Cyan).bold());
        let title_bottom =
            Line::from(" Use the arrow keys to navigate, Enter to select, and Esc to cancel ")
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

    fn render_welcome(&self, area: Rect, buf: &mut Buffer) {
        let title =
            Line::from("  fetchall | Welcome  ").style(Style::default().fg(Color::Cyan).bold());
        let welcome_text = vec![
            Line::from(""),
            Line::from(""),
            Line::styled(
                "Welcome to Fetchall CLI Setup!",
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
    fn render_new_profile(&self, area: Rect, buf: &mut Buffer, text: &str, cursor: usize) {
        let title: Line<'_> =
            Line::from("  fetchall | New Profile  ").style(Style::default().fg(Color::Cyan).bold());
        let title_bottom =
            Line::from(" Type a profile name, Enter to create, and Esc to cancel ".italic())
                .centered()
                .style(Style::default().fg(Color::DarkGray));
        let cur_name = if cursor < text.len() {
            Line::from(vec![
                Span::styled(&text[..cursor], Style::default().fg(Color::Yellow)),
                Span::styled(
                    &text[cursor..cursor.saturating_add(1)],
                    Style::default().bg(Color::LightYellow).fg(Color::Black),
                ),
                Span::styled(
                    &text[cursor.saturating_add(1)..],
                    Style::default().fg(Color::Yellow),
                ),
                Span::from(" "),
            ])
        } else {
            Line::from(vec![
                Span::styled(text, Style::default().fg(Color::Yellow)),
                Span::styled(
                    " ",
                    Style::default().bg(Color::LightYellow).fg(Color::Black),
                ),
            ])
        };
        let lines = vec![
            Line::from(""),
            Line::from("Please enter a name for the new profile:"),
            Line::from(""),
            cur_name,
            Line::from(""),
            Line::from(""),
        ];
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

    fn render_rename_profile(
        &self,
        area: Rect,
        buf: &mut Buffer,
        text: &str,
        profile: &str,
        cursor: usize,
    ) {
        let title: Line<'_> = Line::from(format!("  fetchall | Rename {}  ", profile))
            .style(Style::default().fg(Color::Cyan).bold());
        let title_bottom =
            Line::from(" Edit the profile name, Enter to confirm, and Esc to cancel ".italic())
                .centered()
                .style(Style::default().fg(Color::DarkGray));
        let cur_name = if cursor < text.len() {
            Line::from(vec![
                Span::styled(&text[..cursor], Style::default().fg(Color::Yellow)),
                Span::styled(
                    &text[cursor..cursor.saturating_add(1)],
                    Style::default().bg(Color::LightYellow).fg(Color::Black),
                ),
                Span::styled(
                    &text[cursor.saturating_add(1)..],
                    Style::default().fg(Color::Yellow),
                ),
                Span::from(" "),
            ])
        } else {
            Line::from(vec![
                Span::styled(text, Style::default().fg(Color::Yellow)),
                Span::styled(
                    " ",
                    Style::default().bg(Color::LightYellow).fg(Color::Black),
                ),
            ])
        };
        let lines = vec![
            Line::from(""),
            Line::from(format!("Please enter a new name for {}:", profile)),
            Line::from(""),
            cur_name,
            Line::from(""),
            Line::from(""),
        ];
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

    fn render_integrations_menu(
        &self,
        area: Rect,
        buf: &mut Buffer,
        selected: usize,
        gh_status: (bool, bool),
        hacka_status: bool,
    ) {
        let lines = vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "GitHub",
                    if selected == 0 {
                        Style::default().fg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
                Span::from("  "),
                if gh_status.0 {
                    if gh_status.1 {
                        Span::styled("[ CONNECTED ]", Style::default().fg(Color::Green))
                    } else {
                        Span::styled("[ IN PROGRESS ]", Style::default().fg(Color::Yellow))
                    }
                } else {
                    Span::styled("[ NOT CONNECTED ]", Style::default().fg(Color::Red))
                },
            ]),
            Line::from(vec![
                Span::styled(
                    "Hackatime",
                    if selected == 1 {
                        Style::default().fg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
                Span::from("  "),
                if hacka_status {
                    Span::styled("[ CONNECTED ]", Style::default().fg(Color::Green))
                } else {
                    Span::styled("[ NOT CONNECTED ]", Style::default().fg(Color::Red))
                },
            ]),
            Line::from(""),
            Line::styled(
                "Back",
                if selected == 2 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Line::from(""),
            Line::from(""),
        ];
        let title = Line::from("  fetchall | Integrations  ")
            .style(Style::default().fg(Color::Cyan).bold());
        let title_bottom =
            Line::from(" Use Enter to authenticate, D to revoke, and Esc to exit. ".italic())
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

    fn render_select_file(
        &self,
        area: Rect,
        buf: &mut Buffer,
        text: &str,
        cursor: usize,
        msg: &str,
    ) {
        let title: Line<'_> = Line::from("  fetchall | Logo | Select ASCII Art  ")
            .style(Style::default().fg(Color::Cyan).bold());
        let title_bottom =
            Line::from(" Type the file's path, Enter to confirm, and Esc to cancel ".italic())
                .centered()
                .style(Style::default().fg(Color::DarkGray));
        let cur_name = if cursor < text.len() {
            Line::from(vec![
                Span::styled(&text[..cursor], Style::default().fg(Color::Yellow)),
                Span::styled(
                    &text[cursor..cursor.saturating_add(1)],
                    Style::default().bg(Color::LightYellow).fg(Color::Black),
                ),
                Span::styled(
                    &text[cursor.saturating_add(1)..],
                    Style::default().fg(Color::Yellow),
                ),
                Span::from(" "),
            ])
        } else {
            Line::from(vec![
                Span::styled(text, Style::default().fg(Color::Yellow)),
                Span::styled(
                    " ",
                    Style::default().bg(Color::LightYellow).fg(Color::Black),
                ),
            ])
        };
        let msg = if msg.lines().count() > 1 {
            let mut t = vec![Line::styled(
                msg.lines().next().unwrap_or(""),
                Style::default().fg(Color::Red).bold(),
            )];
            t.extend(msg.lines().skip(1).map(Line::from).collect::<Vec<Line>>());
            t
        } else {
            vec![Line::from(msg)]
        };
        let mut lines = vec![Line::from("")];
        lines.extend(msg.clone());
        lines.extend(vec![
            Line::from(""),
            cur_name,
            Line::from(""),
            Line::from(""),
        ]);
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

    fn render_profile_editor(&self, area: Rect, buf: &mut Buffer, profile: &str, selected: usize) {
        let lines: Vec<Line<'_>> = vec![
            Line::from(""),
            Line::from(""),
            Line::styled(
                "Modify Components",
                if selected == 0 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Line::from(""),
            Line::styled(
                "Rename Profile",
                if selected == 1 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Line::styled(
                "Delete Profile",
                if selected == 2 {
                    Style::default().fg(Color::LightRed).bold()
                } else {
                    Style::default().fg(Color::Red)
                },
            ),
            Line::from(""),
            Line::styled(
                "Back",
                if selected == 3 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Line::from(""),
            Line::from(""),
        ];
        let title = Line::from(format!("  fetchall | {}  ", profile))
            .style(Style::default().fg(Color::Cyan).bold());
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

    fn render_appearance(&self, area: Rect, buf: &mut Buffer, selected: usize) {
        let lines = vec![
            Line::from(""),
            Line::from(""),
            Line::styled(
                "Logo",
                if selected == 0 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Line::styled(
                "Colour Scheme",
                if selected == 1 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Line::styled(
                "Layout",
                if selected == 2 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Line::from(""),
            Line::styled(
                "Back",
                if selected == 3 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Line::from(""),
            Line::from(""),
        ];
        let title =
            Line::from("  fetchall | Appearance  ").style(Style::default().fg(Color::Cyan).bold());
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

    fn render_profiles(&self, area: Rect, buf: &mut Buffer, selected: usize) {
        let mut lines = vec![
            Line::from(""),
            Line::styled(
                "Select a profile to modify",
                Style::default().fg(Color::LightCyan).bold(),
            ),
            Line::from(""),
            Line::from(""),
        ];
        for profile in config::get_profiles().iter() {
            lines.push(Line::styled(
                profile.to_string(),
                if selected
                    == config::get_profiles()
                        .iter()
                        .position(|p| p == profile)
                        .unwrap_or(0)
                {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ))
        }
        lines.push(Line::from(""));
        lines.push(Line::styled(
            "Create New Profile",
            if selected == config::get_profiles().len() {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::White)
            },
        ));
        lines.push(Line::styled(
            "Reset to defaults",
            if selected == config::get_profiles().len() + 1 {
                Style::default().fg(Color::LightRed).bold()
            } else {
                Style::default().fg(Color::Red)
            },
        ));
        lines.push(Line::from(""));
        lines.push(Line::styled(
            "Back",
            if selected == config::get_profiles().len() + 2 {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
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

    fn get_ordered_components(profile: &str) -> Vec<&(&str, &str)> {
        let config = config::load_config(None).unwrap();
        let enabled = config
            .profiles
            .get(&profile.to_string().to_lowercase())
            .map(|f| f.components.clone())
            .unwrap_or_default();
        [
            enabled
                .iter()
                .map(|c| {
                    COMPONENTS
                        .iter()
                        .find(|comp| comp.0 == c)
                        .unwrap_or(&("", ""))
                })
                .collect::<Vec<_>>(),
            COMPONENTS
                .iter()
                .filter(|a| !enabled.contains(&a.0.to_string()))
                .collect::<Vec<_>>(),
        ]
        .concat()
    }

    fn render_edit_components(
        &self,
        area: Rect,
        buf: &mut Buffer,
        profile: &str,
        selected: usize,
        moving: Option<usize>,
    ) {
        let config = config::load_config(None).unwrap();
        let enabled = config
            .profiles
            .get(&profile.to_string().to_lowercase())
            .map(|f| f.components.clone())
            .unwrap_or_default();

        let index = if let Some(moving) = moving {
            moving
        } else {
            selected
        };
        let highlight = if moving.is_some() {
            Style::default().fg(Color::LightYellow).bold().italic()
        } else {
            Style::default().fg(Color::Yellow).bold()
        };

        let mut lines: Vec<Line<'_>> = vec![Line::from(""), Line::from("")];
        for (i, component) in Self::get_ordered_components(profile).iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(
                    component.1,
                    if index == i {
                        highlight
                    } else if enabled.contains(&component.0.to_string()) {
                        Style::default().fg(Color::White)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::from("\n"),
                Span::styled(
                    if enabled.contains(&component.0.to_string()) {
                        " [ ON ]"
                    } else {
                        "  [ OFF ]"
                    },
                    if enabled.contains(&component.0.to_string()) {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    },
                ),
            ]));
        }
        lines.push(Line::from(""));
        lines.push(Line::styled(
            "Return",
            if selected == component::COMPONENTS.len() {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::White)
            },
        ));
        lines.push(Line::from(""));
        lines.push(Line::from(""));

        let title = Line::from(format!("  fetchall | {} | Choose Components  ", profile))
            .style(Style::default().fg(Color::Cyan).bold());
        let title_bottom = Line::from(
            " Use arrow keys to navigate, Enter to toggle, M to reorder, and Esc to exit. "
                .italic(),
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

    fn render_os_logos(&self, area: Rect, buf: &mut Buffer, selected: usize) {
        let current = config::load_config(None).unwrap().appearance.logo;
        let mut lines = vec![Line::from(""), Line::from("")];
        for os in logo::OS_LOGOS.iter() {
            lines.push(Line::styled(
                os.to_string(),
                if selected == logo::OS_LOGOS.iter().position(|p| p == os).unwrap_or(0) {
                    if &current == os {
                        Style::default().fg(Color::LightGreen).bold().italic()
                    } else {
                        Style::default().fg(Color::Yellow).bold()
                    }
                } else {
                    if &current == os {
                        Style::default().fg(Color::Green).italic()
                    } else {
                        Style::default().fg(Color::White)
                    }
                },
            ))
        }
        lines.push(Line::from(""));
        lines.push(Line::styled(
            "Back",
            if selected == logo::OS_LOGOS.len() {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        let title = Line::from("  fetchall | Logo | OS Override  ")
            .style(Style::default().fg(Color::Cyan).bold());
        let title_bottom = Line::from(
            " Use arrow keys to navigate, Enter to select an OS, and Esc to exit. ".italic(),
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

    fn render_logo_menu(&self, area: Rect, buf: &mut Buffer, selected: usize) {
        let current = config::load_config(None).unwrap().appearance.logo;
        let lines = vec![
            Line::from(""),
            Line::from(""),
            Line::styled(
                "Automatic (OS Detection)",
                if selected == 0 {
                    if current == "auto" {
                        Style::default().fg(Color::LightGreen).bold().italic()
                    } else {
                        Style::default().fg(Color::Yellow).bold()
                    }
                } else {
                    if current == "auto" {
                        Style::default().fg(Color::Green).italic()
                    } else {
                        Style::default().fg(Color::White)
                    }
                },
            ),
            Line::styled(
                "Override OS Logo",
                if selected == 1 {
                    if logo::OS_LOGOS.contains(&current.to_lowercase().as_str()) {
                        Style::default().fg(Color::LightGreen).bold().italic()
                    } else {
                        Style::default().fg(Color::Yellow)
                    }
                } else {
                    if logo::OS_LOGOS.contains(&current.to_lowercase().as_str()) {
                        Style::default().fg(Color::Green).italic()
                    } else {
                        Style::default().fg(Color::White)
                    }
                },
            ),
            Line::styled(
                "Fetchall Logo",
                if selected == 2 {
                    if current == "fetchall" {
                        Style::default().fg(Color::LightGreen).bold().italic()
                    } else {
                        Style::default().fg(Color::Yellow).bold()
                    }
                } else {
                    if current == "fetchall" {
                        Style::default().fg(Color::Green).italic()
                    } else {
                        Style::default().fg(Color::White)
                    }
                },
            ),
            Line::styled(
                "Custom ASCII File",
                if selected == 3 {
                    if current != "auto"
                        && current != "fetchall"
                        && !logo::OS_LOGOS.contains(&current.to_lowercase().as_str())
                    {
                        Style::default().fg(Color::LightGreen).bold().italic()
                    } else {
                        Style::default().fg(Color::Yellow).bold()
                    }
                } else {
                    if current != "auto"
                        && current != "fetchall"
                        && current != "none"
                        && !logo::OS_LOGOS.contains(&current.to_lowercase().as_str())
                    {
                        Style::default().fg(Color::Green).italic()
                    } else {
                        Style::default().fg(Color::White)
                    }
                },
            ),
            Line::styled(
                "None",
                if selected == 4 {
                    if current == "none" {
                        Style::default().fg(Color::LightGreen).bold().italic()
                    } else {
                        Style::default().fg(Color::Yellow).bold()
                    }
                } else {
                    if current == "none" {
                        Style::default().fg(Color::Green).italic()
                    } else {
                        Style::default().fg(Color::White)
                    }
                },
            ),
            Line::from(""),
            Line::styled(
                "Back",
                if selected == 5 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Line::from(""),
            Line::from(""),
        ];
        let title =
            Line::from("  fetchall | Logo Config  ").style(Style::default().fg(Color::Cyan).bold());
        let title_bottom =
            Line::from(" Use arrow keys to navigate, Enter to select, and Esc to exit ".italic())
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

    fn render_layout_menu(&self, area: Rect, buf: &mut Buffer, selected: usize) {
        let config = config::load_config(None).unwrap();
        let lines = vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Show section titles",
                    if selected == 0 {
                        Style::default().fg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
                Span::from("  "),
                Span::styled(
                    if config.appearance.show_section_titles {
                        "[ ON ]"
                    } else {
                        "[ OFF ]"
                    },
                    if config.appearance.show_section_titles {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Logo gap:",
                    if selected == 1 {
                        Style::default().fg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
                Span::from("  "),
                Span::styled(
                    format!("{}", config.appearance.logo_gap),
                    if selected == 1 {
                        Style::default().fg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Label gap:",
                    if selected == 2 {
                        Style::default().fg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
                Span::from("  "),
                Span::styled(
                    format!("{}", config.appearance.label_gap),
                    if selected == 2 {
                        Style::default().fg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Gap between sections",
                    if selected == 3 {
                        Style::default().fg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
                Span::from("  "),
                Span::styled(
                    if config.appearance.show_section_gap {
                        "[ ON ]"
                    } else {
                        "[ OFF ]"
                    },
                    if config.appearance.show_section_gap {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    },
                ),
            ]),
            Line::from(""),
            Line::styled(
                "Back",
                if selected == 4 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Line::from(""),
            Line::from(""),
        ];
        let title =
            Line::from("  fetchall | Layout  ").style(Style::default().fg(Color::Cyan).bold());
        let title_bottom = Line::from(
            " Use Up/Down to navigate, Left/Right to adjust, Enter to toggle, and Esc to exit. "
                .italic(),
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

    fn render_colour_menu(&self, area: Rect, buf: &mut Buffer, selected: usize) {
        let lines = vec![
            Line::from(""),
            Line::from(""),
            Line::styled(
                "Text Colours",
                if selected == 0 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Line::styled(
                "Logo Colours",
                if selected == 1 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Line::from(""),
            Line::styled(
                "Reset to defaults",
                if selected == 2 {
                    Style::default().fg(Color::LightRed)
                } else {
                    Style::default().fg(Color::Red)
                },
            ),
            Line::from(""),
            Line::styled(
                "Back",
                if selected == 3 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Line::from(""),
            Line::from(""),
        ];
        let title = Line::from("  fetchall | Colour Config  ")
            .style(Style::default().fg(Color::Cyan).bold());
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

    fn render_main_menu(&self, area: Rect, buf: &mut Buffer, selected: usize) {
        let lines: Vec<Line<'_>> = vec![
            Line::from(""),
            Line::from(""),
            Line::styled(
                "Profiles",
                if selected == 0 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            // Line::styled(
            //     "Components",
            //     if selected == 1 {
            //         Style::default().fg(Color::Yellow).bold()
            //     } else {
            //         Style::default().fg(Color::White)
            //     },
            // ),
            Line::styled(
                "Appearance",
                if selected == 1 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Line::styled(
                "Integrations",
                if selected == 2 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Line::from(""),
            Line::styled(
                "Exit",
                if selected == 3 {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::DarkGray)
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
        match &self.screen {
            ConfigScreens::Welcome => self.render_welcome(area, buf),
            ConfigScreens::MainMenu { selected } => self.render_main_menu(area, buf, *selected),
            ConfigScreens::ProfilesMenu { selected } => self.render_profiles(area, buf, *selected),
            ConfigScreens::ProfileEditor { profile, selected } => {
                self.render_profile_editor(area, buf, profile, *selected)
            }
            ConfigScreens::EnableComponentsMenu {
                profile,
                selected,
                moving,
            } => self.render_edit_components(area, buf, profile, *selected, *moving),
            ConfigScreens::NewProfileMenu { name, cursor } => {
                self.render_new_profile(area, buf, name, *cursor)
            }
            ConfigScreens::RenameProfileMenu {
                profile,
                name,
                cursor,
            } => self.render_rename_profile(area, buf, name, profile, *cursor),
            ConfigScreens::AppearanceMenu { selected } => {
                self.render_appearance(area, buf, *selected)
            }
            ConfigScreens::LogoMenu { selected } => self.render_logo_menu(area, buf, *selected),
            ConfigScreens::FileInputMenu {
                input,
                cursor,
                title,
            } => self.render_select_file(area, buf, input, *cursor, title),
            ConfigScreens::OsLogoMenu { selected } => self.render_os_logos(area, buf, *selected),
            ConfigScreens::ColourMenu { selected } => self.render_colour_menu(area, buf, *selected),
            ConfigScreens::LogoColourMenu { selected } => {
                self.render_logo_colour(area, buf, *selected)
            }
            ConfigScreens::MainColourMenu { selected } => {
                self.render_colour_choice_menu(area, buf, *selected)
            }
            ConfigScreens::LayoutMenu { selected } => self.render_layout_menu(area, buf, *selected),
            ConfigScreens::ComponentsMenu => todo!(),
            ConfigScreens::IntegrationsMenu {
                selected,
                hac_status,
                gh_status,
            } => self.render_integrations_menu(area, buf, *selected, *gh_status, *hac_status),
            ConfigScreens::HackatimeLoginScreen { done1, done2, err } => {
                self.render_hackatime_login(area, buf, *done1, *done2, err)
            }
            ConfigScreens::GitHubLoginScreen {
                done1,
                done2,
                err,
                url,
                code,
            } => self.render_github_login(area, buf, *done1, *done2, code, url, err),
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
                    github_auth::revoke(false)?;
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
                    hackatime_auth::revoke(false)?;
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
            UserComponent::new(&config::UserConfig::default(), "auto", &false, &false)
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
