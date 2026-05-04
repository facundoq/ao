use super::common::{SystemCommand, is_completing_arg};
use crate::cli::{UserAction, UserArgs};
use crate::os::{Domain, ExecutableCommand, OutputFormat, UserInfo, UserManager, UserSessionInfo};
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
            Some(UserAction::List {
                all,
                groups,
                format,
            }) => self.list(*all, *groups, *format),
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
            Some(UserAction::Delete { username, purge }) => self.delete(username, *purge),
            Some(UserAction::Modify {
                username,
                action,
                value,
            }) => self.modify_user(username, action, value),
            Some(UserAction::Passwd { username }) => self.passwd(username),
            Some(UserAction::Session {
                username,
                all,
                n,
                format,
            }) => self.session(username.as_deref(), *all, *n, *format),
            None => self.list(false, false, OutputFormat::Table),
        }
    }
    fn complete(
        &self,
        _line: &str,
        words: &[&str],
        last_word_complete: bool,
    ) -> Result<Vec<String>> {
        if is_completing_arg(words, &["ao", "user", "delete"], 1, last_word_complete)
            || is_completing_arg(words, &["ao", "user", "modify"], 1, last_word_complete)
            || is_completing_arg(words, &["ao", "user", "passwd"], 1, last_word_complete)
        {
            return self.get_users();
        }

        if is_completing_arg(words, &["ao", "user", "modify"], 2, last_word_complete) {
            return Ok(vec![
                "add-group".to_string(),
                "del-group".to_string(),
                "shell".to_string(),
                "home".to_string(),
            ]);
        }

        if is_completing_arg(words, &["ao", "user", "modify"], 3, last_word_complete) {
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
    fn list(
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

    fn delete(&self, username: &str, purge: bool) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(UserDelCommand {
            username: username.to_string(),
            purge,
        }))
    }

    fn modify_user(
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

    fn session(
        &self,
        username: Option<&str>,
        all: bool,
        n: Option<u32>,
        format: OutputFormat,
    ) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(UserSessionCommand {
            username: username.map(|s| s.to_string()),
            all,
            n,
            format,
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

        // Sort: Regular first, then System.
        users.sort_by(|a, b| {
            if a.user_type == b.user_type {
                // If same type, sort by UID numerically
                let a_uid: u32 = a.uid.parse().unwrap_or(0);
                let b_uid: u32 = b.uid.parse().unwrap_or(0);
                a_uid.cmp(&b_uid)
            } else if a.user_type == "Regular" {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Username", "Type", "UID", "GID", "Home", "Shell"]);
                for u in users {
                    table.add_row(vec![u.username, u.user_type, u.uid, u.gid, u.home, u.shell]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&users)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&users)?);
            }
            OutputFormat::Original => unreachable!(),
        }
        Ok(())
    }
    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }
    fn as_string(&self) -> String {
        "cat /etc/passwd".to_string()
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
                .arg(&self.value)
                .arg("--")
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
                .arg(&self.value)
                .arg("--")
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
    fn as_string(&self) -> String {
        format!("chpasswd (for user {})", self.username)
    }
}

pub struct UserSessionCommand {
    pub username: Option<String>,
    pub all: bool,
    pub n: Option<u32>,
    pub format: OutputFormat,
}

impl ExecutableCommand for UserSessionCommand {
    fn execute(&self) -> Result<()> {
        let mut cmd = Command::new("last");
        cmd.arg("--time-format").arg("iso");

        if let Some(n) = self.n {
            cmd.arg("-n").arg(n.to_string());
        }

        if let Some(ref u) = self.username {
            cmd.arg(u);
        } else if !self.all {
            // Get current user using whoami
            let whoami_output = Command::new("whoami").output()?;
            let current_user = String::from_utf8_lossy(&whoami_output.stdout)
                .trim()
                .to_string();
            if !current_user.is_empty() {
                cmd.arg(current_user);
            }
        }

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut sessions = Vec::new();
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("wtmp begins") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }

            let username = parts[0].to_string();
            let tty = parts[1].to_string();

            // Find the first part that looks like an ISO date (contains 'T' and '-')
            let mut start_idx = None;
            for (i, part) in parts.iter().enumerate().skip(2) {
                if part.contains('T')
                    && part.contains('-')
                    && part.chars().next().unwrap_or(' ').is_ascii_digit()
                {
                    start_idx = Some(i);
                    break;
                }
            }

            let start_idx = match start_idx {
                Some(idx) => idx,
                None => continue,
            };

            let start_time = parts[start_idx].to_string();

            // Host is between TTY and start time
            let tty_pos = line.find(&tty).unwrap() + tty.len();
            let start_pos = line.find(&start_time).unwrap();
            let host = line[tty_pos..start_pos].trim().to_string();

            let mut end_time = String::new();
            let mut duration = String::new();

            if start_idx + 2 < parts.len() && parts[start_idx + 1] == "-" {
                let next_part = parts[start_idx + 2];
                if next_part.contains('T') && next_part.contains('-') {
                    end_time = next_part.to_string();
                    // Duration is usually the last part, e.g. (00:00)
                    if let Some(last_part) = parts.last() {
                        duration = last_part.trim_matches(|c| c == '(' || c == ')').to_string();
                    }
                }
            }

            if end_time.is_empty() {
                if line.contains("still logged in") {
                    end_time = "still logged in".to_string();
                } else if line.contains("gone - no logout") {
                    end_time = "gone - no logout".to_string();
                } else if line.contains("crash") {
                    end_time = "crash".to_string();
                } else if line.contains("down") {
                    end_time = "down".to_string();
                }
            }

            sessions.push(UserSessionInfo {
                username,
                line: tty,
                host,
                start: start_time,
                end: end_time,
                duration,
            });
        }

        sessions.reverse();

        match self.format {
            OutputFormat::Table => {
                use colored::Colorize;
                let mut table = comfy_table::Table::new();
                table.set_header(vec![
                    "User",
                    "TTY",
                    "Host",
                    "Start Date",
                    "Start Time",
                    "End Date",
                    "End Time",
                    "Duration",
                ]);

                let split_date_time = |dt: &str| -> (String, String) {
                    if dt.contains('T') {
                        let parts: Vec<&str> = dt.split('T').collect();
                        let date = parts[0].to_string();
                        let time_tz = parts[1];
                        let time =
                            if let Some(pos) = time_tz.find('+').or_else(|| time_tz.find('-')) {
                                time_tz[..pos].to_string()
                            } else {
                                time_tz.to_string()
                            };
                        (date, time)
                    } else {
                        (dt.to_string(), String::new())
                    }
                };

                for s in sessions {
                    let is_active = s.end == "still logged in";
                    let is_error =
                        s.end == "crash" || s.end == "down" || s.end == "gone - no logout";

                    let (s_date_str, s_time_str) = split_date_time(&s.start);
                    let (e_date_str, e_time_str) = split_date_time(&s.end);

                    let (start_date, start_time) = if is_active {
                        (s_date_str.green(), s_time_str.green())
                    } else if is_error {
                        (s_date_str.red(), s_time_str.red())
                    } else {
                        (s_date_str.yellow(), s_time_str.yellow())
                    };

                    let (end_date, end_time) = if is_active {
                        (e_date_str.green().bold(), e_time_str.green().bold())
                    } else if is_error {
                        (e_date_str.red().bold(), e_time_str.red().bold())
                    } else {
                        (e_date_str.yellow(), e_time_str.yellow())
                    };

                    let duration = s.duration.yellow();

                    table.add_row(vec![
                        s.username.normal(),
                        s.line.normal(),
                        s.host.dimmed(),
                        start_date,
                        start_time,
                        end_date,
                        end_time,
                        duration,
                    ]);
                }
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&sessions)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&sessions)?);
            }
            OutputFormat::Original => {
                println!("{}", stdout);
            }
        }
        Ok(())
    }

    fn is_structured(&self) -> bool {
        matches!(
            self.format,
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Original
        )
    }

    fn as_string(&self) -> String {
        let mut s = "last --time-format iso".to_string();
        if let Some(n) = self.n {
            s.push_str(&format!(" -n {}", n));
        }
        if let Some(ref u) = self.username {
            s.push_str(&format!(" {}", u));
        }
        s
    }
}
