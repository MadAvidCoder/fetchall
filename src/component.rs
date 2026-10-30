use crate::components::{
    dev_env::DevEnvComponent, git::GitComponent, hackatime::HackatimeComponent,
    system::SystemComponent, user::UserComponent, weather::WeatherComponent,
};
use crate::config::Config;
use crate::output::ComponentOutput;
use anyhow::Result;

pub const COMPONENTS: &[(&str, &str)] = &[
    ("system", "System Stats"),
    ("git", "Git Repo Info"),
    ("dev_env", "Development Environment"),
    ("github_repo", "GitHub Repository"),
    ("github_profile", "GitHub Profile"),
    ("user", "Username"),
    ("hackatime", "Hackatime Stats"),
    ("weather", "Today's Weather"),
];

pub trait Component {
    fn title(&self) -> &str;
    fn collect(&self) -> Result<ComponentOutput>;
}

pub fn get_component_by_name<'a>(
    name: &'a str,
    config: &'a Config,
) -> Option<Box<dyn Component + 'a>> {
    match name {
        "system" => Some(Box::new(SystemComponent::new(
            &config.components.system,
            &config.appearance.colour,
            &config.appearance.show_section_titles,
        ))),
        "git" => Some(Box::new(GitComponent::new(
            &config.components.git,
            &config.appearance.colour,
            &config.appearance.show_section_titles,
        ))),
        "dev_env" => Some(Box::new(DevEnvComponent::new(
            &config.components.dev_env,
            &config.appearance.colour,
            &config.appearance.show_section_titles,
        ))),
        "github_repo" => Some(Box::new(
            crate::components::github_repo::GithubRepoComponent::new(
                &config.components.github_repo,
                &config.appearance.colour,
                &config.appearance.show_section_titles,
            ),
        )),
        "github_profile" => Some(Box::new(
            crate::components::github_profile::GithubProfileComponent::new(
                &config.components.github_profile,
                &config.appearance.colour,
                &config.appearance.show_section_titles,
            ),
        )),
        "user" => Some(Box::new(UserComponent::new(
            &config.components.user,
            &config.appearance.colour,
            &config.appearance.show_section_titles,
        ))),
        "hackatime" => Some(Box::new(HackatimeComponent::new(
            &config.components.hackatime,
            &config.appearance.colour,
            &config.appearance.show_section_titles,
        ))),
        "weather" => Some(Box::new(WeatherComponent::new(
            &config.components.weather,
            &config.appearance.colour,
            &config.appearance.show_section_titles,
        ))),
        _ => None,
    }
}
