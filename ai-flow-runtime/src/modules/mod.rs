mod client;
mod company;
mod conversation;
mod registry;
mod smalltalk;
mod traits;
mod types;

pub use client::ClientModuleFactory;
pub use company::CompanyModuleFactory;
pub use conversation::ConversationModuleFactory;
pub use registry::ModuleRegistry;
pub use smalltalk::SmallTalkModuleFactory;
pub use traits::{Module, ModuleFactory, ModuleInstance};
pub use types::{ModuleContent, ModuleInfo, ModuleRef};

pub const CLIENT_MODULE_ID: &str = "client";
pub const COMPANY_MODULE_ID: &str = "company";
pub const CONVERSATION_MODULE_ID: &str = "conversation";
pub const SMALLTALK_MODULE_ID: &str = "smalltalk";

pub fn default_registry() -> ModuleRegistry {
    let mut registry = ModuleRegistry::new();
    registry.register(Box::new(ClientModuleFactory::new()));
    registry.register(Box::new(CompanyModuleFactory::new()));
    registry.register(Box::new(ConversationModuleFactory::new()));
    registry.register(Box::new(SmallTalkModuleFactory::new()));
    registry
}
