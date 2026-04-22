use anyhow::{Context, Result};
use clap::{CommandFactory, FromArgMatches};
use cli::{Cli, CliCommand};
use os::Domain;

pub mod cli;
pub mod config;
pub mod os;

fn main() -> Result<()> {
    // 0. Load config
    let config = config::Config::load().unwrap_or_default();

    // 1. Detect the system
    let system = os::detector::detect_system()?;
    let mut domains = system.domains();
    domains.sort_by_key(|d| d.name());

    // 2. Build the command tree dynamically
    let mut app = Cli::command();
    for domain in &domains {
        app = app.subcommand(domain.command());
    }

    // Hide help from completions globally
    app = hide_help_globally(app);

    // 3. Handle dynamic completions if requested (intercepts execution)
    if handle_dynamic_completion(&domains, &app)? {
        return Ok(());
    }

    // 4. Parse arguments
    let matches = app.clone().get_matches();
    let cli = Cli::from_arg_matches(&matches).context("Failed to parse global flags")?;

    if let Some(CliCommand::Interactive) = cli.command {
        return run_interactive(&domains);
    }

    if cli.dump_tree {
        dump_command_tree(&app, "", true, true);
        return Ok(());
    }

    // 5. Find the matching domain and execute
    let (subcommand_name, subcommand_matches) = match matches.subcommand() {
        Some(s) => s,
        None => {
            app.print_help()?;
            println!();
            return Ok(());
        }
    };

    let domain = domains
        .iter()
        .find(|d| d.name() == subcommand_name)
        .with_context(|| format!("Unknown command: {}", subcommand_name))?;

    let executable = domain.execute(subcommand_matches, &app)?;

    if cli.print {
        executable.print()?;
    } else if cli.dry_run {
        executable.dry_run()?;
    } else {
        // Print the command used to obtain the output by default
        if config.ui.show_command && !executable.is_structured() {
            use colored::Colorize;
            println!(
                "{} {}",
                ">".blue().bold(),
                executable.as_string().bright_black().italic()
            );
        }
        executable.execute()?;
    }

    Ok(())
}

fn run_interactive(domains: &[&dyn Domain]) -> Result<()> {
    use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};

    let mut domain_list = domains.to_vec();
    domain_list.sort_by_key(|d| d.name());

    let mut domain_names: Vec<String> = domain_list.iter().map(|d| d.name().to_string()).collect();
    domain_names.push("exit".to_string());

    let theme = ColorfulTheme::default();

    loop {
        let selection = Select::with_theme(&theme)
            .with_prompt("Select a domain")
            .items(&domain_names)
            .default(0)
            .interact()?;

        if domain_names[selection] == "exit" {
            break;
        }

        let domain = domain_list[selection];
        let mut cmd = domain.command();
        let mut args_vec = vec!["ao".to_string(), domain.name().to_string()];

        let mut aborted = false;

        loop {
            let subcommands: Vec<_> = cmd.get_subcommands().collect();
            if subcommands.is_empty() {
                break; // leaf node
            }

            let mut sub_names: Vec<String> = subcommands
                .iter()
                .filter(|s| s.get_name() != "help")
                .map(|s| s.get_name().to_string())
                .collect();

            if sub_names.is_empty() {
                break;
            }

            sub_names.push("<back>".to_string());

            let selection = Select::with_theme(&theme)
                .with_prompt(format!("Select action for {}", cmd.get_name()))
                .items(&sub_names)
                .default(0)
                .interact()?;

            let selected_name = &sub_names[selection];
            if selected_name == "<back>" {
                aborted = true;
                break;
            }

            args_vec.push(selected_name.to_string());
            cmd = subcommands
                .into_iter()
                .find(|s| s.get_name() == selected_name)
                .unwrap()
                .clone();
        }

        if aborted {
            continue;
        }

        // Special handling for some commands to provide better prompts
        let is_user_add = args_vec.len() == 3 && args_vec[1] == "user" && args_vec[2] == "add";

        let mut positional_args = Vec::new();
        let mut option_args = Vec::new();

        for arg in cmd.get_arguments() {
            if arg.get_id() == "help" || arg.get_id() == "version" {
                continue;
            }
            if arg.is_positional() {
                positional_args.push(arg);
            } else {
                option_args.push(arg);
            }
        }

        let mut user_aborted = false;

        for arg in positional_args {
            let help_str;
            let prompt = if is_user_add && arg.get_id() == "username" {
                "Enter the unique system username"
            } else {
                help_str = arg
                    .get_help()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| arg.get_id().to_string());
                &help_str
            };

            let val: String = if is_user_add && arg.get_id() == "username" {
                Input::<String>::with_theme(&theme)
                    .with_prompt(prompt)
                    .allow_empty(!arg.is_required_set())
                    .validate_with(|s: &String| -> Result<(), &str> {
                        if s.is_empty() {
                            return Err("Username cannot be empty");
                        }
                        if s.chars()
                            .any(|c| !c.is_alphanumeric() && c != '_' && c != '-')
                        {
                            return Err("Username contains invalid characters");
                        }
                        Ok(())
                    })
                    .interact_text()?
            } else {
                Input::<String>::with_theme(&theme)
                    .with_prompt(prompt)
                    .allow_empty(!arg.is_required_set())
                    .interact_text()?
            };

            if matches!(arg.get_action(), clap::ArgAction::Append) {
                if !val.is_empty() {
                    for v in val.split(',') {
                        args_vec.push(v.trim().to_string());
                    }
                } else if arg.is_required_set() {
                    println!("Argument is required, aborting.");
                    user_aborted = true;
                    break;
                }
            } else if !val.is_empty() {
                args_vec.push(val);
            } else if arg.is_required_set() {
                println!("Argument is required, aborting.");
                user_aborted = true;
                break;
            }
        }

        if user_aborted {
            continue;
        }

        for arg in option_args {
            if arg.get_id() == "format" {
                if let Some(long) = arg.get_long() {
                    args_vec.push(format!("--{}", long));
                } else if let Some(short) = arg.get_short() {
                    args_vec.push(format!("-{}", short));
                }
                args_vec.push("table".to_string());
                continue;
            }

            let help_str;
            let prompt = if is_user_add && arg.get_id() == "name" {
                "User's full name"
            } else if is_user_add && arg.get_id() == "email" {
                "User's email address"
            } else if is_user_add && arg.get_id() == "no_create_home" {
                "Should the home directory be created?" // We'll invert this for the prompt
            } else {
                help_str = arg
                    .get_help()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| arg.get_id().to_string());
                &help_str
            };

            if matches!(arg.get_action(), clap::ArgAction::SetTrue) {
                let enable = if is_user_add && arg.get_id() == "no_create_home" {
                    // Invert for "Should create home?"
                    let create = Confirm::with_theme(&theme)
                        .with_prompt(prompt)
                        .default(true)
                        .interact()?;
                    !create
                } else {
                    Confirm::with_theme(&theme)
                        .with_prompt(prompt)
                        .default(false)
                        .interact()?
                };

                if enable {
                    if let Some(long) = arg.get_long() {
                        args_vec.push(format!("--{}", long));
                    } else if let Some(short) = arg.get_short() {
                        args_vec.push(format!("-{}", short));
                    }
                }
            } else if matches!(arg.get_action(), clap::ArgAction::Set) {
                let mut extra_prompt = String::new();
                let possible_vals: Vec<_> = arg
                    .get_possible_values()
                    .iter()
                    .map(|v| v.get_name().to_string())
                    .collect();
                if !possible_vals.is_empty() {
                    extra_prompt = format!(" [{}]", possible_vals.join(", "));
                }

                let mut default_val = String::new();
                if arg.get_id() == "format" {
                    default_val = "table".to_string();
                }

                let val: String = if is_user_add && arg.get_id() == "email" {
                    let input = Input::<String>::with_theme(&theme);
                    let input = input.with_prompt(format!("{}{}", prompt, extra_prompt));
                    let input = if !default_val.is_empty() {
                        input.with_initial_text(default_val)
                    } else {
                        input
                    };
                    input
                        .allow_empty(!arg.is_required_set())
                        .validate_with(|s: &String| -> Result<(), &str> {
                            if !s.is_empty() && !s.contains('@') {
                                return Err("Invalid email format");
                            }
                            Ok(())
                        })
                        .interact_text()?
                } else {
                    let input = Input::<String>::with_theme(&theme);
                    let input = input.with_prompt(format!("{}{}", prompt, extra_prompt));
                    let input = if !default_val.is_empty() {
                        input.with_initial_text(default_val)
                    } else {
                        input
                    };
                    input.allow_empty(!arg.is_required_set()).interact_text()?
                };

                if !val.is_empty() {
                    if let Some(long) = arg.get_long() {
                        args_vec.push(format!("--{}", long));
                    } else if let Some(short) = arg.get_short() {
                        args_vec.push(format!("-{}", short));
                    }
                    args_vec.push(val);
                } else if arg.is_required_set() {
                    println!("Argument is required, aborting.");
                    user_aborted = true;
                    break;
                }
            }
        }

        if user_aborted {
            continue;
        }

        println!("\nExecuting: {}\n", args_vec.join(" "));

        let app = Cli::command();
        match app.try_get_matches_from(&args_vec) {
            Ok(matches) => {
                let (subcommand_name, subcommand_matches) = matches.subcommand().unwrap();
                let domain = domains
                    .iter()
                    .find(|d| d.name() == subcommand_name)
                    .unwrap();

                match domain.execute(subcommand_matches, &Cli::command()) {
                    Ok(executable) => {
                        let config = crate::config::Config::load().unwrap_or_default();
                        if config.ui.show_command && !executable.is_structured() {
                            use colored::Colorize;
                            println!(
                                "{} {}",
                                ">".blue().bold(),
                                executable.as_string().bright_black().italic()
                            );
                        }
                        if let Err(e) = executable.execute() {
                            eprintln!("Error executing command: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Error preparing command: {}", e),
                }
            }
            Err(e) => {
                eprintln!("Error parsing generated command: {}", e);
            }
        }
        println!();
    }

    Ok(())
}

fn handle_dynamic_completion(domains: &[&dyn Domain], app: &clap::Command) -> Result<bool> {
    let ao_complete = std::env::var_os("AO_COMPLETE");
    let clap_complete_index = std::env::var_os("_CLAP_COMPLETE_INDEX");
    let complete_env = std::env::var_os("COMPLETE");

    // Only proceed if we are being asked for completions via our bridge or clap's native protocol
    if ao_complete.is_none() && clap_complete_index.is_none() && complete_env.is_none() {
        return Ok(false);
    }

    if let Some(shell_val) = ao_complete {
        let args: Vec<String> = std::env::args().collect();
        if let Some(dash_dash_idx) = args.iter().position(|a| a == "--") {
            let mut user_words: Vec<String> =
                args.iter().skip(dash_dash_idx + 1).cloned().collect();

            // Normalize first word to "ao" so matching logic works regardless of how binary was called
            if !user_words.is_empty() {
                user_words[0] = "ao".to_string();
            }

            let user_words_refs: Vec<&str> = user_words.iter().map(|s| s.as_str()).collect();
            let line = user_words_refs.join(" ");

            for domain in domains {
                let suggestions = domain.complete(&line, &user_words_refs, false)?;
                if !suggestions.is_empty() {
                    for suggestion in suggestions {
                        if suggestion != "help" && suggestion != "--help" && suggestion != "-h" {
                            println!("{}", suggestion);
                        }
                    }
                    return Ok(true);
                }
            }
        }

        // Handover to clap if manual logic didn't match.
        // We MUST have an index to get suggestions instead of the script.
        if clap_complete_index.is_some() {
            unsafe {
                std::env::set_var("COMPLETE", shell_val);
            }
            let app_clone = app.clone();
            clap_complete::env::CompleteEnv::with_factory(move || app_clone.clone()).complete();
            return Ok(true);
        }
    }

    if std::env::var_os("COMPLETE").is_some() || clap_complete_index.is_some() {
        let app_clone = app.clone();
        clap_complete::env::CompleteEnv::with_factory(move || app_clone.clone()).complete();
        return Ok(true);
    }

    Ok(false)
}

/// Recursively hides the help flag from completions for a command and all its subcommands.
fn hide_help_globally(mut cmd: clap::Command) -> clap::Command {
    if cmd.get_arguments().any(|a| a.get_id() == "help") {
        cmd = cmd.mut_arg("help", |a| a.hide(true));
    }

    let sub_names: Vec<String> = cmd
        .get_subcommands()
        .map(|s| s.get_name().to_string())
        .collect();
    for name in sub_names {
        cmd = cmd.mut_subcommand(&name, hide_help_globally);
    }
    cmd
}

fn dump_command_tree(cmd: &clap::Command, indent: &str, is_last: bool, is_root: bool) {
    let subcommands: Vec<_> = cmd
        .get_subcommands()
        .filter(|s| s.get_name() != "help")
        .collect();
    let sub_count = subcommands.len();

    if is_root {
        println!("{}", cmd.get_name());
    } else {
        let marker = if is_last { "└── " } else { "├── " };
        println!("{}{}{}", indent, marker, cmd.get_name());
    }

    let next_indent = if is_root {
        "".to_string()
    } else {
        format!("{}{}", indent, if is_last { "    " } else { "│   " })
    };

    for (i, sub) in subcommands.into_iter().enumerate() {
        dump_command_tree(sub, &next_indent, i == sub_count - 1, false);
    }
}
