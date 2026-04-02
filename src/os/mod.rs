use anyhow::Result;

pub mod debian;
pub mod detector;

/// Abstracts system package management operations.
pub trait PackageManager {
    fn update(&self) -> Result<()>;
    fn install(&self, packages: &[String]) -> Result<()>;
    fn remove(&self, packages: &[String], purge: bool) -> Result<()>;
    fn search(&self, query: &str) -> Result<()>;
    fn list(&self) -> Result<()>;
}

/// Abstracts system service management operations.
pub trait ServiceManager {
    fn list(&self) -> Result<()>;
    fn up(&self, service: &str) -> Result<()>;
    fn down(&self, service: &str) -> Result<()>;
    fn restart(&self, service: &str) -> Result<()>;
    fn reload(&self, service: &str) -> Result<()>;
    fn status(&self, service: &str) -> Result<()>;
}

/// Abstracts system user management operations.
pub trait UserManager {
    fn list(&self, all: bool, groups: bool) -> Result<()>;
    fn add(&self, username: &str, groups: Option<&str>, shell: Option<&str>, system: bool) -> Result<()>;
    fn del(&self, username: &str, purge: bool) -> Result<()>;
    fn mod_user(&self, username: &str, action: &str, value: &str) -> Result<()>;
    fn passwd(&self, username: &str) -> Result<()>;
}

/// Abstracts system group management operations.
pub trait GroupManager {
    fn list(&self) -> Result<()>;
    fn add(&self, groupname: &str) -> Result<()>;
    fn del(&self, groupname: &str) -> Result<()>;
    fn mod_group(&self, groupname: &str, gid: u32) -> Result<()>;
}

/// Abstracts system disk management operations.
pub trait DiskManager {
    fn list(&self) -> Result<()>;
    fn mount(&self, device: &str, path: &str, fstype: Option<&str>, options: Option<&str>) -> Result<()>;
    fn unmount(&self, target: &str, lazy: bool, force: bool) -> Result<()>;
    fn usage(&self, path: &str, depth: Option<u32>) -> Result<()>;
}
