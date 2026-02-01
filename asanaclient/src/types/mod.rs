//! Type definitions for the Asana API.

pub mod common;
pub mod custom_field;
pub mod portfolio;
pub mod project;
pub mod status_update;
pub mod story;
pub mod task;
pub mod user;
pub mod workspace;

// Re-export common types at the top level for convenience.
pub use common::{DataWrapper, Gid, ListWrapper, NextPage, ResourceRef, StatusColor, UserRef};
pub use custom_field::{
    extract_status_field, CustomFieldDefinition, CustomFieldSetting, CustomFieldType,
    CustomFieldValue, DateValue, EnumOption, ExtractedStatus, StatusExtractionOptions,
};
pub use portfolio::{Portfolio, PortfolioItem, PortfolioItemRef};
pub use project::Project;
pub use status_update::{StatusUpdate, StatusUpdateRef};
pub use story::{Story, StoryType};
pub use task::{Task, TaskDependency, TaskMembership, TaskRef};
pub use user::{FavoriteItem, User, UserPhoto};
pub use workspace::Workspace;
