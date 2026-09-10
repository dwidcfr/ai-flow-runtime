use crate::error::Result;
use crate::modules::ModuleContent;

pub trait Module: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

pub trait ModuleFactory: Send + Sync {
    fn module(&self) -> &dyn Module;
    fn create_instance(&self) -> Box<dyn ModuleInstance>;
}

pub trait ModuleInstance: Send {
    fn module_id(&self) -> &str;
    fn load(&mut self, source: &str) -> Result<()>;
    fn get_content(&self, section: Option<&str>) -> Result<ModuleContent>;
    fn set_entry(&mut self, key: &str, value: &str) -> Result<()>;
    fn clear(&mut self);
}
