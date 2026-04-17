use anyhow::{Context, Result};
use clap::{Command, CommandFactory, FromArgMatches};
use clap_complete::env::CompleteEnv;
use cli::Cli;
use os::Domain;

pub mod cli;
pub mod config;
pub mod os;

fn main() -> Result<()> {
    // 0. Load config
    let config = config::Config::load().unwrap_or_default();

    // 1. Detect the system first
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

    // 4. Parse arguments using the dynamic tree
    let app_for_parsing = app.clone();
    let matches = app_for_parsing.get_matches();
    let cli = Cli::from_arg_matches(&matches).context("Failed to parse global flags")?;

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

fn handle_dynamic_completion(domains: &[&dyn Domain], app: &Command) -> Result<bool> {
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
            CompleteEnv::with_factory(move || app_clone.clone()).complete();
            return Ok(true);
        }
    }

    if std::env::var_os("COMPLETE").is_some() || clap_complete_index.is_some() {
        let app_clone = app.clone();
        CompleteEnv::with_factory(move || app_clone.clone()).complete();
        return Ok(true);
    }

    Ok(false)
}

/// Recursively hides the help flag from completions for a command and all its subcommands.
fn hide_help_globally(mut cmd: Command) -> Command {
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
