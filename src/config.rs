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
                components: vec![
                    "user".to_string(),
                    "system".to_string(),
                    "weather".to_string(),
                ],
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
                    "github_repo".to_string(),
                    "dev_env".to_string(),
                ],
            },
        );
        profiles.insert(
            "life".to_string(),
            ProfileConfig {
                components: vec![
                    "user".to_string(),
                    "github_profile".to_string(),
                    "hackatime".to_string(),
                    "weather".to_string(),
                ],
            },
        );
        // profiles.insert(
        //     "all".to_string(),
        //     ProfileConfig {
        //         components: vec![
        //             "user".to_string(),
        //             "system".to_string(),
        //             "git".to_string(),
        //             "dev_env".to_string(),
        //             "github_repo".to_string(),
        //             "github_profile".to_string(),
        //             "hackatime".to_string(),
        //             "weather".to_string(),
        //         ],
        //     },
        // );
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
    pub show_section_gap: bool,
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
            show_section_gap: true,
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
    pub show_uptime: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        SystemConfig {
            enabled: true,
            show_os: true,
            show_memory: true,
            show_disk: true,
            show_cpu: true,
            show_uptime: true,
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

pub fn rename_profile(old_name: &str, new_name: &str) -> Result<()> {
    let mut config = load_config(None)?;
    if let Some(profile_entry) = config.profiles.remove(&old_name.to_lowercase()) {
        config
            .profiles
            .insert(new_name.to_string().to_lowercase(), profile_entry);
        save_config(&config, None)?;
    }
    Ok(())
}

pub fn add_profile(profile: &str) -> Result<()> {
    let mut config = load_config(None)?;
    if !config.profiles.contains_key(&profile.to_lowercase()) {
        config.profiles.insert(
            profile.to_string().to_lowercase(),
            ProfileConfig {
                components: vec!["user".to_string(), "system".to_string(), "git".to_string()],
            },
        );
        save_config(&config, None)?;
    }
    Ok(())
}
pub fn move_component_prof(profile: &str, component: &str, new_index: usize) -> Result<()> {
    let mut config = load_config(None)?;
    if let Some(profile_entry) = config.profiles.get_mut(&profile.to_string().to_lowercase())
        && let Some(current_index) = profile_entry
            .components
            .iter()
            .position(|c| c == &component.to_string().to_lowercase())
        && new_index < profile_entry.components.len()
    {
        let component = profile_entry.components.remove(current_index);
        profile_entry.components.insert(new_index, component);
    }
    save_config(&config, None)?;
    Ok(())
}

pub fn remove_profile(profile: &str) -> Result<()> {
    let mut config = load_config(None)?;
    if config.profiles.remove(&profile.to_lowercase()).is_some() {
        save_config(&config, None)?;
    }
    Ok(())
}

pub fn add_component_to_profile(profile: &str, component: &str) -> Result<()> {
    let mut config = load_config(None)?;
    let profile_entry = config
        .profiles
        .entry(profile.to_string().to_lowercase())
        .or_insert(ProfileConfig::default());
    if !profile_entry
        .components
        .contains(&component.to_string().to_lowercase())
    {
        profile_entry
            .components
            .push(component.to_string().to_lowercase());
    }
    save_config(&config, None)?;
    Ok(())
}

pub fn remove_component_from_profile(profile: &str, component: &str) -> Result<()> {
    let mut config = load_config(None)?;
    if let Some(profile_entry) = config.profiles.get_mut(&profile.to_string().to_lowercase()) {
        profile_entry
            .components
            .retain(|c| c != &component.to_string().to_lowercase());
    }
    save_config(&config, None)?;
    Ok(())
}

pub fn reset_profiles() -> Result<()> {
    let mut config = load_config(None)?;
    config.profiles.clear();
    config.profiles = Config::default().profiles;
    save_config(&config, None)?;
    Ok(())
}

fn save_config(config: &Config, path: Option<&Path>) -> Result<()> {
    let config_path = match path {
        Some(p) => p.to_path_buf(),
        None => {
            let project_dirs = ProjectDirs::from("", "", "fetchall")
                .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
            project_dirs.config_dir().join("config.toml")
        }
    };
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let config_content = toml::to_string_pretty(config)?;
    std::fs::write(&config_path, config_content)?;
    Ok(())
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
        config.profiles = config_loaded.profiles;
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

pub fn get_profiles() -> Vec<String> {
    let config = load_config(None).unwrap_or_default();
    let mut profiles: Vec<String> = config.profiles.keys().cloned().collect();
    profiles.sort();
    profiles
}

pub fn set_logo(logo: &str) -> Result<()> {
    let mut config = load_config(None)?;
    config.appearance.logo = logo.to_string();
    save_config(&config, None)?;
    Ok(())
}

pub fn set_logo_colour(colour: &str) -> Result<()> {
    let mut config = load_config(None)?;
    config.appearance.logo_colour = colour.to_string();
    save_config(&config, None)?;
    Ok(())
}

pub fn set_colour(colour: &str) -> Result<()> {
    let mut config = load_config(None)?;
    config.appearance.colour = colour.to_string();
    save_config(&config, None)?;
    Ok(())
}

pub fn set_logo_gap(gap: usize) -> Result<()> {
    let mut config = load_config(None)?;
    config.appearance.logo_gap = gap;
    save_config(&config, None)?;
    Ok(())
}

pub fn set_label_gap(gap: usize) -> Result<()> {
    let mut config = load_config(None)?;
    config.appearance.label_gap = gap;
    save_config(&config, None)?;
    Ok(())
}

pub fn set_title_visibility(visible: bool) -> Result<()> {
    let mut config = load_config(None)?;
    config.appearance.show_section_titles = visible;
    save_config(&config, None)?;
    Ok(())
}

pub fn set_section_gap(visible: bool) -> Result<()> {
    let mut config = load_config(None)?;
    config.appearance.show_section_gap = visible;
    save_config(&config, None)?;
    Ok(())
}
