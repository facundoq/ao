use anyhow::Result;

pub mod debian;
pub mod detector;

/// Represents a command that can be executed, printed, or dry-run.
pub trait ExecutableCommand {
    fn execute(&self) -> Result<()>;
    fn dry_run(&self) -> Result<()>;
    fn print(&self) -> Result<()>;
    fn as_string(&self) -> String;
}

/// Abstracts system package management operations.
pub trait PackageManager {
    fn update(&self) -> Result<Box<dyn ExecutableCommand>>;
    fn install(&self, packages: &[String]) -> Result<Box<dyn ExecutableCommand>>;
    fn remove(&self, packages: &[String], purge: bool) -> Result<Box<dyn ExecutableCommand>>;
    fn search(&self, query: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn list(&self) -> Result<Box<dyn ExecutableCommand>>;
}

/// Abstracts system service management operations.
pub trait ServiceManager {
    fn list(&self) -> Result<Box<dyn ExecutableCommand>>;
    fn up(&self, service: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn down(&self, service: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn restart(&self, service: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn reload(&self, service: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn status(&self, service: &str) -> Result<Box<dyn ExecutableCommand>>;
}

/// Abstracts system user management operations.
pub trait UserManager {
    fn list(&self, all: bool, groups: bool) -> Result<Box<dyn ExecutableCommand>>;
    fn add(
        &self,
        username: &str,
        groups: Option<&str>,
        shell: Option<&str>,
        system: bool,
    ) -> Result<Box<dyn ExecutableCommand>>;
    fn del(&self, username: &str, purge: bool) -> Result<Box<dyn ExecutableCommand>>;
    fn mod_user(
        &self,
        username: &str,
        action: &str,
        value: &str,
    ) -> Result<Box<dyn ExecutableCommand>>;
    fn passwd(&self, username: &str) -> Result<Box<dyn ExecutableCommand>>;
}

/// Abstracts system group management operations.
pub trait GroupManager {
    fn list(&self) -> Result<Box<dyn ExecutableCommand>>;
    fn add(&self, groupname: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn del(&self, groupname: &str) -> Result<Box<dyn ExecutableCommand>>;
    fn mod_group(&self, groupname: &str, gid: u32) -> Result<Box<dyn ExecutableCommand>>;
}

/// Abstracts system disk management operations.
pub trait DiskManager {
    fn list(&self) -> Result<Box<dyn ExecutableCommand>>;
    fn mount(
        &self,
        device: &str,
        path: &str,
        fstype: Option<&str>,
        options: Option<&str>,
    ) -> Result<Box<dyn ExecutableCommand>>;
    fn unmount(&self, target: &str, lazy: bool, force: bool) -> Result<Box<dyn ExecutableCommand>>;
    fn usage(&self, path: &str, depth: Option<u32>) -> Result<Box<dyn ExecutableCommand>>;
}

/// Abstracts system monitoring operations.
pub trait MonitorManager {
    fn live_stats(&self) -> Result<()>;
}
