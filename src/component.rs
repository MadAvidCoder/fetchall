use crate::output::ComponentOutput;
use anyhow::Result;

pub trait Component {
    fn title(&self) -> &str;
    fn collect(&self) -> Result<ComponentOutput>;
}
