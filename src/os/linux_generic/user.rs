use super::common::{SystemCommand, is_completing_arg};
use crate::cli::{UserAction, UserArgs};
use crate::os::{Domain, ExecutableCommand, OutputFormat, UserInfo, UserManager};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use std::process::Command;

pub struct StandardUser;

impl Domain for StandardUser {
    fn name(&self) -> &'static str {
        "user"
    }
    fn command(&self) -> ClapCommand {
        UserArgs::augment_args(ClapCommand::new("user").about("Manage users"))
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        _app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = UserArgs::from_arg_matches(matches)?;
        match &args.action {
            Some(UserAction::Ls {
                all,
                groups,
                format,
            }) => self.ls(*all, *groups, *format),
            Some(UserAction::Add {
                username,
                name,
                email,
                groups,
                shell,
                system,
                no_create_home,
            }) => self.add(
                username,
                name.as_deref(),
                email.as_deref(),
                groups.as_deref(),
                shell.as_deref(),
                *system,
                *no_create_home,
            ),
            Some(UserAction::Del { username, purge }) => self.del(username, *purge),
            Some(UserAction::Mod {
                username,
                action,
                value,
            }) => self.mod_user(username, action, value),
            Some(UserAction::Passwd { username }) => self.passwd(username),
            None => self.ls(false, false, OutputFormat::Table),
        }
    }
    fn complete(
        &self,
        _line: &str,
        words: &[&str],
        last_word_complete: bool,
    ) -> Result<Vec<String>> {
        if is_completing_arg(words, &["ao", "user", "del"], 1, last_word_complete)
            || is_completing_arg(words, &["ao", "user", "mod"], 1, last_word_complete)
            || is_completing_arg(words, &["ao", "user", "passwd"], 1, last_word_complete)
        {
            return self.get_users();
        }

        if is_completing_arg(words, &["ao", "user", "mod"], 2, last_word_complete) {
            return Ok(vec![
                "add-group".to_string(),
                "del-group".to_string(),
                "shell".to_string(),
                "home".to_string(),
            ]);
        }

        if is_completing_arg(words, &["ao", "user", "mod"], 3, last_word_complete) {
            let action = words.get(words.len() - 2).copied().unwrap_or("");
            if action == "add-group" || action == "del-group" {
                let output = Command::new("cut")
                    .arg("-d:")
                    .arg("-f1")
                    .arg("/etc/group")
                    .output()?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Ok(stdout.lines().map(|s| s.trim().to_string()).collect());
            } else if action == "shell" {
                return self.get_shells();
            }
        }
        Ok(vec![])
    }
}

impl UserManager for StandardUser {
    fn ls(
        &self,
        all: bool,
        groups: bool,
        format: OutputFormat,
    ) -> Result<Box<dyn ExecutableCommand>> {
        if matches!(format, OutputFormat::Original) {
            return Ok(Box::new(SystemCommand::new("cat").arg("/etc/passwd")));
        }
        Ok(Box::new(UserListCommand {
            all,
            groups,
            format,
        }))
    }

    fn add(
        &self,
        username: &str,
        name: Option<&str>,
        email: Option<&str>,
        groups: Option<&str>,
        shell: Option<&str>,
        system: bool,
        no_create_home: bool,
    ) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(UserAddCommand {
            username: username.to_string(),
            name: name.map(|s| s.to_string()),
            email: email.map(|s| s.to_string()),
            groups: groups.map(|s| s.to_string()),
            shell: shell.map(|s| s.to_string()),
            system,
            no_create_home,
        }))
    }

    fn del(&self, username: &str, purge: bool) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(UserDelCommand {
            username: username.to_string(),
            purge,
        }))
    }

    fn mod_user(
        &self,
        username: &str,
        action: &str,
        value: &str,
    ) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(UserModCommand {
            username: username.to_string(),
            action: action.to_string(),
            value: value.to_string(),
        }))
    }

    fn passwd(&self, username: &str) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(PasswdCommand {
            username: username.to_string(),
        }))
    }

    fn get_users(&self) -> Result<Vec<String>> {
        let output = Command::new("cut")
            .arg("-d:")
            .arg("-f1")
            .arg("/etc/passwd")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    fn get_shells(&self) -> Result<Vec<String>> {
        let output = Command::new("cat").arg("/etc/shells").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .map(|s| s.trim().to_string())
            .collect())
    }
}

pub struct UserListCommand {
    pub all: bool,
    pub groups: bool,
    pub format: OutputFormat,
}
impl ExecutableCommand for UserListCommand {
    fn execute(&self) -> Result<()> {
        let output = Command::new("cat").arg("/etc/passwd").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut users = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 7 {
                let uid: u32 = parts[2].parse().unwrap_or(0);
                if !self.all && uid < 1000 && uid != 0 {
                    continue;
                }

                let user_type = if uid == 0 || uid >= 1000 {
                    "Regular"
                } else {
                    "System"
                };

                users.push(UserInfo {
                    username: parts[0].to_string(),
                    uid: parts[2].to_string(),
                    gid: parts[3].to_string(),
                    home: parts[5].to_string(),
                    shell: parts[6].to_string(),
                    groups: Vec::new(),
                    user_type: user_type.to_string(),
                });
            }
        }

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Username", "Type", "UID", "GID", "Home", "Shell"]);
                for u in users {
                    table.add_row(vec![u.username, u.user_type, u.uid, u.gid, u.home, u.shell]);
                }
                println!("{}", table);
            }
            _ => self.format.print_structured(&users)?,
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] List users (format: {:?})", self.format);
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("list users (format: {:?})", self.format);
        Ok(())
    }
    fn as_string(&self) -> String {
        "cat /etc/passwd".to_string()
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
}

pub struct UserAddCommand {
    pub username: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub groups: Option<String>,
    pub shell: Option<String>,
    pub system: bool,
    pub no_create_home: bool,
}
impl ExecutableCommand for UserAddCommand {
    fn execute(&self) -> Result<()> {
        let mut cmd = SystemCommand::new("useradd");
        if !self.no_create_home {
            cmd = cmd.arg("-m");
        }
        if self.system {
            cmd = cmd.arg("--system");
        }
        if let Some(ref s) = self.shell {
            cmd = cmd.arg("--shell").arg(s);
        }
        if let Some(ref g) = self.groups {
            cmd = cmd.arg("--groups").arg(g);
        }

        let mut comment = Vec::new();
        if let Some(ref n) = self.name {
            comment.push(n.clone());
        }
        if let Some(ref e) = self.email {
            comment.push(format!("<{}>", e));
        }
        if !comment.is_empty() {
            cmd = cmd.arg("-c").arg(&comment.join(" "));
        }

        cmd.arg("--").arg(&self.username).execute()
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
        let mut s = "useradd".to_string();
        if !self.no_create_home {
            s.push_str(" -m");
        }
        if self.system {
            s.push_str(" --system");
        }
        if let Some(ref sh) = self.shell {
            s.push_str(&format!(" --shell {}", sh));
        }
        if let Some(ref g) = self.groups {
            s.push_str(&format!(" --groups {}", g));
        }

        let mut comment = Vec::new();
        if let Some(ref n) = self.name {
            comment.push(n.clone());
        }
        if let Some(ref e) = self.email {
            comment.push(format!("<{}>", e));
        }
        if !comment.is_empty() {
            s.push_str(&format!(" -c \"{}\"", comment.join(" ")));
        }

        s.push_str(&format!(" -- {}", self.username));
        s
    }
}

pub struct UserDelCommand {
    pub username: String,
    pub purge: bool,
}
impl ExecutableCommand for UserDelCommand {
    fn execute(&self) -> Result<()> {
        let mut cmd = SystemCommand::new("userdel");
        if self.purge {
            cmd = cmd.arg("-r");
        }
        cmd.arg("--").arg(&self.username).execute()
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
        if self.purge {
            format!("userdel -r -- {}", self.username)
        } else {
            format!("userdel -- {}", self.username)
        }
    }
}

pub struct UserModCommand {
    pub username: String,
    pub action: String,
    pub value: String,
}
impl ExecutableCommand for UserModCommand {
    fn execute(&self) -> Result<()> {
        match self.action.as_str() {
            "add-group" => SystemCommand::new("usermod")
                .arg("-aG")
                .arg("--")
                .arg(&self.value)
                .arg(&self.username)
                .execute(),
            "del-group" => SystemCommand::new("gpasswd")
                .arg("-d")
                .arg(&self.username)
                .arg("--")
                .arg(&self.value)
                .execute(),
            "shell" => SystemCommand::new("usermod")
                .arg("-s")
                .arg("--")
                .arg(&self.value)
                .arg(&self.username)
                .execute(),
            "home" => SystemCommand::new("usermod")
                .arg("-d")
                .arg(&self.value)
                .arg("-m")
                .arg("--")
                .arg(&self.username)
                .execute(),
            _ => anyhow::bail!("Unsupported user modification action: {}", self.action),
        }
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
        match self.action.as_str() {
            "add-group" => format!("usermod -aG {} -- {}", self.value, self.username),
            "del-group" => format!("gpasswd -d {} -- {}", self.username, self.value),
            "shell" => format!("usermod -s {} -- {}", self.value, self.username),
            "home" => format!("usermod -d {} -m -- {}", self.value, self.username),
            _ => format!("usermod (invalid action) -- {}", self.username),
        }
    }
}

pub struct PasswdCommand {
    pub username: String,
}

impl ExecutableCommand for PasswdCommand {
    fn execute(&self) -> Result<()> {
        let password = rpassword::prompt_password("New password: ")?;
        let confirm = rpassword::prompt_password("Retype new password: ")?;
        if password != confirm {
            anyhow::bail!("Passwords do not match");
        }
        let creds = format!("{}:{}", self.username, password);
        SystemCommand::new("chpasswd").stdin(&creds).execute()
    }
    fn dry_run(&self) -> Result<()> {
        println!("[DRY RUN] chpasswd for user {}", self.username);
        Ok(())
    }
    fn print(&self) -> Result<()> {
        println!("chpasswd (for user {})", self.username);
        Ok(())
    }
    fn as_string(&self) -> String {
        format!("chpasswd (for user {})", self.username)
    }
}
