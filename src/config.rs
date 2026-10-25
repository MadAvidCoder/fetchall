use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
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
                components: vec!["system".to_string(), "git".to_string()],
            },
        );
        profiles.insert(
            "system".to_string(),
            ProfileConfig {
                components: vec!["system".to_string()],
            },
        );
        profiles.insert(
            "dev".to_string(),
            ProfileConfig {
                components: vec!["git".to_string(), "dev_env".to_string()],
            },
        );
        profiles.insert("life".to_string(), ProfileConfig { components: vec![] });
        profiles.insert("game".to_string(), ProfileConfig { components: vec![] });
        Config {
            general: GeneralConfig::default(),
            appearance: AppearanceConfig::default(),
            components: ComponentsConfig::default(),
            profiles,
        }
    }
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct AppearanceConfig {
    pub logo: String,
    pub logo_color: String,
    pub color: String,
    pub show_section_titles: bool,
    pub logo_gap: usize,
    pub label_gap: usize,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        AppearanceConfig {
            logo: "auto".to_string(),
            logo_color: "auto".to_string(),
            color: "auto".to_string(),
            show_section_titles: true,
            logo_gap: 4,
            label_gap: 2,
        }
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct ComponentsConfig {
    pub system: SystemConfig,
    pub dev_env: DevEnvConfig,
    pub git: GitConfig,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct ProfileConfig {
    pub components: Vec<String>,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        ProfileConfig {
            components: vec!["system".to_string(), "git".to_string()],
        }
    }
}
