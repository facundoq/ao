use anyhow::Result;

pub mod debian;
pub mod fedora;
pub mod arch;
pub mod systemd;
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
