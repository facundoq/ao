use crate::os::{Domain, ExecutableCommand, MonitorManager};
use anyhow::Result;
use clap::{ArgMatches, Command as ClapCommand};
use sysinfo::{Components, Disks, Networks, System};

pub struct StandardMonitor;

impl Domain for StandardMonitor {
    fn name(&self) -> &'static str {
        "monitor"
    }
    fn command(&self) -> ClapCommand {
        ClapCommand::new("monitor").about("Monitor live system stats")
    }
    fn execute(
        &self,
        _matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        self.live_stats()
    }
}

impl MonitorManager for StandardMonitor {
    fn live_stats(&self) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(LiveStatsCommand))
    }
}

pub struct LiveStatsCommand;

impl ExecutableCommand for LiveStatsCommand {
    fn execute(&self) -> Result<()> {
        let mut sys = System::new_all();
        std::thread::sleep(std::time::Duration::from_millis(200));
        sys.refresh_all();
        println!("=== System Monitor ===");
        println!("CPU usage: {:.1}%", sys.global_cpu_usage());
        let components = Components::new_with_refreshed_list();
        for comp in &components {
            if (comp.label().to_lowercase().contains("cpu")
                || comp.label().to_lowercase().contains("core"))
                && comp.temperature().is_some()
            {
                println!(
                    "Temperature ({}): {:.1}°C",
                    comp.label(),
                    comp.temperature().unwrap()
                );
            }
        }
        println!("RAM: {} / {} bytes", sys.used_memory(), sys.total_memory());
        let networks = Networks::new_with_refreshed_list();
        for (name, data) in &networks {
            println!(
                "{}: RX {} bytes, TX {} bytes",
                name,
                data.total_received(),
                data.total_transmitted()
            );
        }
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            println!(
                "{:?}: {} / {} bytes",
                disk.name(),
                disk.total_space() - disk.available_space(),
                disk.total_space()
            );
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] Live stats");
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("Live stats monitor");
        Ok(())
    }
    fn as_string(&self) -> String {
        "live_stats".to_string()
    }
}
