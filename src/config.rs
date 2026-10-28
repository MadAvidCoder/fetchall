use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub appearance: AppearanceConfig,
    pub components: ComponentsConfig,
    pub profiles: HashMap<String, ProfileConfig>,
}
impl Default for Config {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert(
            "default".to_string(),
            ProfileConfig {
                components: vec!["user".to_string(), "system".to_string(), "git".to_string()],
            },
        );
        profiles.insert(
            "system".to_string(),
            ProfileConfig {
                components: vec!["user".to_string(), "system".to_string()],
            },
        );
        profiles.insert(
            "dev".to_string(),
            ProfileConfig {
                components: vec![
                    "user".to_string(),
                    "git".to_string(),
                    "dev_env".to_string(),
                    "github_repo".to_string(),
                    "github_profile".to_string(),
                ],
            },
        );
        profiles.insert(
            "life".to_string(),
            ProfileConfig {
                components: vec![
                    "user".to_string(),
                    "hackatime".to_string(),
                    "weather".to_string(),
                ],
            },
        );
        profiles.insert(
            "game".to_string(),
            ProfileConfig {
                components: vec!["user".to_string()],
            },
        );
        profiles.insert(
            "all".to_string(),
            ProfileConfig {
                components: vec![
                    "user".to_string(),
                    "system".to_string(),
                    "git".to_string(),
                    "dev_env".to_string(),
                    "github_repo".to_string(),
                    "github_profile".to_string(),
                    "hackatime".to_string(),
                    "weather".to_string(),
                ],
            },
        );
        Config {
            general: GeneralConfig::default(),
            appearance: AppearanceConfig::default(),
            components: ComponentsConfig::default(),
            profiles,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct GeneralConfig {
    pub profile: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        GeneralConfig {
            profile: "default".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct AppearanceConfig {
    pub logo: String,
    pub logo_colour: String,
    pub colour: String,
    pub show_section_titles: bool,
    pub logo_gap: usize,
    pub label_gap: usize,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        AppearanceConfig {
            logo: "auto".to_string(),
            logo_colour: "auto".to_string(),
            colour: "auto".to_string(),
            show_section_titles: true,
            logo_gap: 4,
            label_gap: 2,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct ComponentsConfig {
    pub system: SystemConfig,
    pub dev_env: DevEnvConfig,
    pub git: GitConfig,
    pub user: UserConfig,
    pub github_repo: GithubRepoConfig,
    pub github_profile: GithubProfileConfig,
    pub hackatime: HackatimeConfig,
    pub weather: WeatherConfig,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct WeatherConfig {
    pub enabled: bool,
    pub show_current: bool,
    pub show_today: bool,
    pub show_wind: bool,
    pub show_sun: bool,
}
impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_current: true,
            show_today: true,
            show_wind: true,
            show_sun: true,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct HackatimeConfig {
    pub enabled: bool,
    pub show_hours: bool,
    pub show_streak: bool,
    pub show_languages: bool,
    pub show_project: bool,
}
impl Default for HackatimeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_hours: true,
            show_streak: true,
            show_languages: true,
            show_project: true,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct GithubRepoConfig {
    pub enabled: bool,
    pub show_url: bool,
    pub show_repo: bool,
    pub show_prs: bool,
    pub show_actions: bool,
    pub show_issues: bool,
    pub show_forks: bool,
    pub show_stars: bool,
}
impl Default for GithubRepoConfig {
    fn default() -> Self {
        GithubRepoConfig {
            enabled: true,
            show_url: false,
            show_repo: true,
            show_prs: true,
            show_actions: true,
            show_issues: true,
            show_forks: false,
            show_stars: true,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct GithubProfileConfig {
    pub enabled: bool,
    pub show_username: bool,
    pub show_repos: bool,
    pub show_followers: bool,
    pub show_prs: bool,
    pub show_commits: bool,
}
impl Default for GithubProfileConfig {
    fn default() -> Self {
        GithubProfileConfig {
            enabled: true,
            show_username: true,
            show_repos: true,
            show_followers: true,
            show_prs: true,
            show_commits: true,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct UserConfig {
    pub enabled: bool,
    pub show_user: bool,
    pub show_hostname: bool,
}

impl Default for UserConfig {
    fn default() -> Self {
        UserConfig {
            enabled: true,
            show_user: true,
            show_hostname: true,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct SystemConfig {
    pub enabled: bool,
    pub show_os: bool,
    pub show_memory: bool,
    pub show_disk: bool,
    pub show_cpu: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        SystemConfig {
            enabled: true,
            show_os: true,
            show_memory: true,
            show_disk: true,
            show_cpu: true,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct DevEnvConfig {
    pub enabled: bool,
    pub show_project: bool,
    pub show_path: bool,
    pub show_project_tools: bool,
    pub show_system_tools: bool,
}

impl Default for DevEnvConfig {
    fn default() -> Self {
        DevEnvConfig {
            enabled: true,
            show_project: true,
            show_path: false,
            show_project_tools: true,
            show_system_tools: true,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct GitConfig {
    pub enabled: bool,
    pub show_repo: bool,
    pub show_branch: bool,
    pub show_changes: bool,
    pub show_commit: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        GitConfig {
            enabled: true,
            show_repo: true,
            show_branch: true,
            show_changes: true,
            show_commit: true,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct ProfileConfig {
    pub components: Vec<String>,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        ProfileConfig {
            components: vec!["user".to_string(), "system".to_string(), "git".to_string()],
        }
    }
}

pub fn load_config(path: Option<&Path>) -> Result<Config> {
    let config_path = match path {
        Some(p) => p.to_path_buf(),
        None => {
            let project_dirs = ProjectDirs::from("", "", "fetchall")
                .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
            project_dirs.config_dir().join("config.toml")
        }
    };
    if config_path.exists() {
        let mut config = Config::default();
        let config_content = std::fs::read_to_string(&config_path)?;
        let config_loaded: Config = toml::from_str(&config_content)?;
        config.general = config_loaded.general;
        config.appearance = config_loaded.appearance;
        config.components = config_loaded.components;
        config.profiles.extend(config_loaded.profiles);
        Ok(config)
    } else {
        let config = Config::default();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let config_content = toml::to_string_pretty(&config)?;
        std::fs::write(&config_path, config_content)?;
        Ok(config)
    }
}
