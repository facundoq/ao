# Interactive Mode Implementation Status

## Goal
Implement `ao interactive`, a global feature that starts an interactive session to browse domains, move into subdomains, and execute commands with guided argument collection and validation.

## Current State
- [x] Add `Interactive` variant to `CliCommand` in `src/cli.rs`.
- [x] Full implementation of `run_interactive` in `src/main.rs`.
- [x] Domain selection using `dialoguer`.
- [x] Subcommand browsing using `clap` reflection.
- [x] Positional and option/flag argument gathering using `dialoguer`.
- [x] Handling of `ArgAction::SetTrue` (flags), `ArgAction::Set` (options), and `ArgAction::Append` (positional lists).
- [x] Command execution after argument gathering.
- [x] Basic "Back" navigation in subcommand selection.
- [x] **Enhanced `UserAdd`**: Updated `UserAction::Add` and its implementation to include:
    - `name` (full name)
    - `email`
    - Option for home directory creation (inverted prompt for better UX).
- [x] **Improved Validation**: Added basic validation for username (alphanumeric) and email (@ check) in `ao user add`.
- [x] **Help Integration**: Prompts now use the help text from `clap` definitions.

## Missing / To Do
- [ ] **Complex Validation**: More robust validation for other commands.
- [ ] **Session Persistence**: allow history or saving common commands.
- [ ] **Advanced Argument Types**: Handle more complex `clap` argument configurations if any.

## Completed
1. Update `src/cli.rs` with expanded `UserAction::Add`.
2. Update `src/os/linux_generic/user.rs` to handle new `UserAction::Add` fields.
3. Refine `run_interactive` in `src/main.rs` to provide better prompts and validation.
