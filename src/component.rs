use crate::components::{
    dev_env::DevEnvComponent, git::GitComponent, system::SystemComponent, user::UserComponent,
};
use crate::config::Config;
use crate::output::ComponentOutput;
use anyhow::Result;

pub trait Component {
    fn title(&self) -> &str;
    fn collect(&self) -> Result<ComponentOutput>;
}

pub fn get_component_by_name<'a>(
    name: &'a str,
    config: &'a Config,
) -> Option<Box<dyn Component + 'a>> {
    match name {
        "system" => Some(Box::new(SystemComponent::new(&config.components.system))),
        "git" => Some(Box::new(GitComponent::new(&config.components.git))),
        "dev_env" => Some(Box::new(DevEnvComponent::new(&config.components.dev_env))),
        "user" => Some(Box::new(UserComponent::new(&config.components.user))),
        _ => None,
    }
}
