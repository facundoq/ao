use super::common::{SystemCommand, is_completing_arg};
use crate::cli::{GroupAction, GroupArgs};
use crate::os::{Domain, ExecutableCommand, GroupInfo, GroupManager, OutputFormat};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use std::process::Command;

pub struct StandardGroup;

impl Domain for StandardGroup {
    fn name(&self) -> &'static str {
        "group"
    }
    fn command(&self) -> ClapCommand {
        GroupArgs::augment_args(ClapCommand::new("group").about("Manage groups"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = GroupArgs::from_arg_matches(matches)?;
        match &args.action {
            GroupAction::Ls { format } => self.ls(*format),
            GroupAction::Add { groupname } => self.add(groupname),
            GroupAction::Del { groupname } => self.del(groupname),
            GroupAction::Mod { groupname, gid } => self.mod_group(groupname, *gid),
        }
    }
    fn complete(
        &self,
        _line: &str,
        words: &[&str],
        last_word_complete: bool,
    ) -> Result<Vec<String>> {
        if is_completing_arg(words, &["ao", "group", "del"], 1, last_word_complete)
            || is_completing_arg(words, &["ao", "group", "mod"], 1, last_word_complete)
        {
            return self.get_groups();
        }
        Ok(vec![])
    }
}

impl GroupManager for StandardGroup {
    fn ls(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        if matches!(format, OutputFormat::Original) {
            return Ok(Box::new(SystemCommand::new("cat").arg("/etc/group")));
        }
        Ok(Box::new(GroupListCommand { format }))
    }
    fn add(&self, groupname: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(GroupAddCommand {
            groupname: groupname.to_string(),
        }))
    }
    fn del(&self, groupname: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(GroupDelCommand {
            groupname: groupname.to_string(),
        }))
    }
    fn mod_group(&self, groupname: &str, gid: u32) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(GroupModCommand {
            groupname: groupname.to_string(),
            gid,
        }))
    }

    fn get_groups(&self) -> Result<Vec<String>> {
        let output = Command::new("cut")
            .arg("-d:")
            .arg("-f1")
            .arg("/etc/group")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.trim().to_string()).collect())
    }
}

pub struct GroupListCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for GroupListCommand {
    fn execute(&self) -> Result<()> {
        let output = Command::new("cat").arg("/etc/group").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut groups = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 4 {
                groups.push(GroupInfo {
                    name: parts[0].to_string(),
                    gid: parts[2].to_string(),
                    members: parts[3]
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect(),
                });
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Group", "GID", "Members"]);
                for g in groups {
                    table.add_row(vec![g.name, g.gid, g.members.join(",")]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&groups)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&groups)?);
            }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] List groups (format: {:?})", self.format);
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("list groups (format: {:?})", self.format);
        Ok(())
    }
    fn as_string(&self) -> String {
        format!("list groups --format {:?}", self.format)
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}

pub struct GroupAddCommand {
    pub groupname: String,
}
impl ExecutableCommand for GroupAddCommand {
    fn execute(&self) -> Result<()> {
        SystemCommand::new("groupadd")
            .arg("--")
            .arg(&self.groupname)
            .execute()
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] {}", self.as_string());
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("{}", self.as_string());
        Ok(())
    }
    fn as_string(&self) -> String {
        format!("groupadd -- {}", self.groupname)
    }
}

pub struct GroupDelCommand {
    pub groupname: String,
}
impl ExecutableCommand for GroupDelCommand {
    fn execute(&self) -> Result<()> {
        SystemCommand::new("groupdel")
            .arg("--")
            .arg(&self.groupname)
            .execute()
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] {}", self.as_string());
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("{}", self.as_string());
        Ok(())
    }
    fn as_string(&self) -> String {
        format!("groupdel -- {}", self.groupname)
    }
}

pub struct GroupModCommand {
    pub groupname: String,
    pub gid: u32,
}
impl ExecutableCommand for GroupModCommand {
    fn execute(&self) -> Result<()> {
        SystemCommand::new("groupmod")
            .arg("--gid")
            .arg(&self.gid.to_string())
            .arg("--")
            .arg(&self.groupname)
            .execute()
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] {}", self.as_string());
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("{}", self.as_string());
        Ok(())
    }
    fn as_string(&self) -> String {
        format!("groupmod --gid {} -- {}", self.gid, self.groupname)
    }
}
