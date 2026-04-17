use super::common::SystemCommand;
use crate::cli::{CompletionsAction, SelfAction, SelfArgs};
use crate::os::{Domain, ExecutableCommand, OutputFormat, SelfInfo, SelfManager};
use anyhow::Result;
use clap::{ArgMatches, Args, Command as ClapCommand, FromArgMatches};
use clap_complete::Shell;
use std::io::Write;

pub struct StandardSelf;

impl Domain for StandardSelf {
    fn name(&self) -> &'static str {
        "self"
    }
    fn command(&self) -> ClapCommand {
        SelfArgs::augment_args(
            ClapCommand::new("self").about("Manage ao itself (completions, info, update)"),
        )
    }
    fn execute(
        &self,
        matches: &ArgMatches,
        app: &ClapCommand,
    ) -> Result<Box<dyn ExecutableCommand>> {
        let args = SelfArgs::from_arg_matches(matches)?;
        let bin_name = app.get_name().to_string();
        let exe_path = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from(&bin_name))
            .to_string_lossy()
            .to_string();

        match &args.action {
            SelfAction::Completions { action } => {
                match action {
                    CompletionsAction::Generate { shell } => match shell {
                        Shell::Bash => {
                            println!(
                                "_{0}() {{
    local cur prev words cword
    _get_comp_words_by_ref -n : cur prev words cword
    
    local completions
    completions=$(env AO_COMPLETE=bash _CLAP_COMPLETE_INDEX=\"$cword\" \"{1}\" -- \"${{words[@]}}\")
    if [ $? -eq 0 ]; then
        COMPREPLY=( $(compgen -W \"${{completions}}\" -- \"${{cur}}\") )
    fi
}}
complete -F _{0} {0}",
                                bin_name, exe_path
                            );
                        }
                        Shell::Zsh => {
                            println!(
                                "_{0}() {{
    local -a completions
    local output
    output=$(AO_COMPLETE=zsh _CLAP_COMPLETE_INDEX=$(($CURRENT - 1)) \"{1}\" -- \"${{words[@]}}\")
    if [ $? -eq 0 ]; then
        completions=(${{(f)output}})
        if [[ \"$completions[1]\" == *:* ]]; then
            _describe 'values' completions
        elif [[ -n \"$completions\" ]]; then
            compadd -a completions
        fi
    fi
}}
compdef _{0} {0}",
                                bin_name, exe_path
                            );
                        }
                        _ => {
                            let mut app_clone = app.clone();
                            clap_complete::generate(
                                *shell,
                                &mut app_clone,
                                bin_name,
                                &mut std::io::stdout(),
                            );
                        }
                    },
                    CompletionsAction::Add { shell } => {
                        self.install_completions(*shell, &exe_path)?;
                    }
                    CompletionsAction::Setup { shell } => match shell {
                        Shell::Bash => println!(
                            "eval \"$(AO_COMPLETE=bash {} self completions generate bash)\"",
                            exe_path
                        ),
                        Shell::Zsh => println!(
                            "eval \"$(AO_COMPLETE=zsh {} self completions generate zsh)\"",
                            exe_path
                        ),
                        Shell::Fish => println!(
                            "AO_COMPLETE=fish {} self completions generate fish | source",
                            exe_path
                        ),
                        _ => eprintln!("Unsupported shell for setup: {:?}", shell),
                    },
                }
                Ok(Box::new(CompletionScriptCommand))
            }
            SelfAction::Info { format } => self.info(*format),
            SelfAction::Update => self.update(),
        }
    }
}

pub struct CompletionScriptCommand;
impl ExecutableCommand for CompletionScriptCommand {
    fn execute(&self) -> Result<()> {
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        Ok(())
    }
    fn print(&self) -> Result<()> {
        Ok(())
    }
    fn as_string(&self) -> String {
        "".to_string()
    }
    fn is_structured(&self) -> bool {
        true
    }
}

impl SelfManager for StandardSelf {
    fn info(&self, format: OutputFormat) -> Result<Box<dyn ExecutableCommand>> {
        Ok(Box::new(SelfInfoCommand { format }))
    }

    fn update(&self) -> Result<Box<dyn ExecutableCommand>> {
        // Placeholder: in a real app, this would check GitHub/a repo
        Ok(Box::new(
            SystemCommand::new("echo").arg("Checking for updates... (not implemented)"),
        ))
    }

    fn install_completions(&self, shell: Shell, exe_path: &str) -> Result<()> {
        use anyhow::Context;
        use std::fs::OpenOptions;

        let home = std::env::var("HOME").context("Failed to find HOME directory")?;
        let (config_path, source_cmd) = match shell {
            Shell::Bash => (
                format!("{}/.bashrc", home),
                format!(
                    "\nsource <(AO_COMPLETE=bash \"{}\" self completions generate bash)\n",
                    exe_path
                ),
            ),
            Shell::Zsh => (
                format!("{}/.zshrc", home),
                format!(
                    "\nsource <(AO_COMPLETE=zsh \"{}\" self completions generate zsh)\n",
                    exe_path
                ),
            ),
            Shell::Fish => (
                format!("{}/.config/fish/config.fish", home),
                format!(
                    "\nAO_COMPLETE=fish \"{}\" self completions generate fish | source\n",
                    exe_path
                ),
            ),
            _ => anyhow::bail!("Auto-installation for {:?} is not supported yet.", shell),
        };

        println!(
            "Installing completions for {:?} in {}...",
            shell, config_path
        );

        if std::fs::read_to_string(&config_path)
            .is_ok_and(|c| c.contains("ao self completions generate"))
        {
            println!("Completions are already installed in {}.", config_path);
            return Ok(());
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config_path)
            .with_context(|| format!("Failed to open {} for writing", config_path))?;

        file.write_all(source_cmd.as_bytes())?;
        println!("Successfully installed completions. Please restart your shell.");
        Ok(())
    }
}

pub struct SelfInfoCommand {
    pub format: OutputFormat,
}
impl ExecutableCommand for SelfInfoCommand {
    fn execute(&self) -> Result<()> {
        let info = SelfInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            os: std::env::consts::OS.to_string(),
        };

        match self.format {
            OutputFormat::Table => {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Property", "Value"]);
                table.add_row(vec!["Version", &info.version]);
                table.add_row(vec!["Architecture", &info.architecture]);
                table.add_row(vec!["OS", &info.os]);
                println!("{}", table);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&info)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&info)?);
            }
            OutputFormat::Original => {
                println!("ao version {}", info.version);
                println!("arch: {}", info.architecture);
                println!("os: {}", info.os);
            }
        }
        Ok(())
    }
    fn dry_run(&self) -> Result<()> {
        self.execute()
    }
    fn print(&self) -> Result<()> {
        println!("ao self info");
        Ok(())
    }
    fn as_string(&self) -> String {
        "ao self info".to_string()
    }
}
