use anyhow::Result;
use std::io::Write;
use clap::{ArgMatches, Command as ClapCommand, FromArgMatches, Args};
use clap_complete::Shell;
use crate::os::{CompletionsManager, ExecutableCommand, Domain};
use crate::cli::{CompletionsArgs, CompletionsAction};
use super::common::NoopCommand;

pub struct StandardCompletions;

impl Domain for StandardCompletions {
    fn name(&self) -> &'static str { "completions" }
    fn command(&self) -> ClapCommand {
        CompletionsArgs::augment_args(ClapCommand::new("completions").about("Shell completion management"))
    }
    fn execute(&self, matches: &ArgMatches, app: &ClapCommand) -> Result<Box<dyn ExecutableCommand>> {
        let args = CompletionsArgs::from_arg_matches(matches)?;
        let bin_name = app.get_name().to_string();
        let exe_path = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from(&bin_name))
            .to_string_lossy()
            .to_string();

        match &args.action {
            CompletionsAction::Generate { shell } => {
                match shell {
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
                        clap_complete::generate(*shell, &mut app_clone, bin_name, &mut std::io::stdout());
                    }
                }
            }
            CompletionsAction::Install { shell } => {
                self.install(*shell, &exe_path)?;
            }
            CompletionsAction::Setup { shell } => {
                match shell {
                    Shell::Bash => println!("eval \"$(AO_COMPLETE=bash {} completions generate bash)\"", exe_path),
                    Shell::Zsh => println!("eval \"$(AO_COMPLETE=zsh {} completions generate zsh)\"", exe_path),
                    Shell::Fish => println!("AO_COMPLETE=fish {} completions generate fish | source", exe_path),
                    _ => eprintln!("Unsupported shell for setup: {:?}", shell),
                }
            }
        }
        Ok(Box::new(NoopCommand))
    }
}

impl CompletionsManager for StandardCompletions {
    fn install(&self, shell: Shell, exe_path: &str) -> Result<()> {
        use std::fs::OpenOptions;
        use anyhow::Context;

        let home = std::env::var("HOME").context("Failed to find HOME directory")?;
        let (config_path, source_cmd) = match shell {
            Shell::Bash => (
                format!("{}/.bashrc", home),
                format!("\nsource <(AO_COMPLETE=bash \"{}\" completions generate bash)\n", exe_path),
            ),
            Shell::Zsh => (
                format!("{}/.zshrc", home),
                format!("\nsource <(AO_COMPLETE=zsh \"{}\" completions generate zsh)\n", exe_path),
            ),
            Shell::Fish => (
                format!("{}/.config/fish/config.fish", home),
                format!("\nAO_COMPLETE=fish \"{}\" completions generate fish | source\n", exe_path),
            ),
            _ => anyhow::bail!("Auto-installation for {:?} is not supported yet.", shell),
        };

        println!("Installing completions for {:?} in {}...", shell, config_path);

        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if content.contains("ao completions generate") {
                println!("Completions are already installed in {}.", config_path);
                return Ok(());
            }
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
