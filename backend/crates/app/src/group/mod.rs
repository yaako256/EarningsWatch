// crates/app/src/group/mod.rs
mod bulk_destination;
mod create_group;
mod delete_group;
pub mod group_config;
mod list_groups;
mod pause_resume_group;
mod test_send;
mod update_group;

pub use bulk_destination::bulk_destination;
pub use create_group::create_group;
pub use delete_group::delete_group;
pub use group_config::{GroupConfigData, get_group_config, put_group_config};
pub use list_groups::list_groups;
pub use pause_resume_group::{pause_group, resume_group};
pub use test_send::{TestSendInput, TestSendOutput, test_send};
pub use update_group::update_group;
