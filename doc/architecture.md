# Project Architecture & Structure

This document outlines the architecture and structure of `ao`, a unified administration tool for Linux systems.

## 1. High-Level Command Execution Flow

`ao` supports two primary execution flows: **Direct Execution** (running commands directly from the shell) and **Interactive Selection** (guided prompting using `dialoguer`).

```mermaid
flowchart TD
    A[Start: User runs ao] --> B{Interactive Command?}
    
    %% Direct execution flow
    B -- No --> C[Detect Host OS via Detector]
    C --> D[Retrieve Domain subcommands & Build Command Tree]
    D --> E[Parse CLI args using Clap]
    E --> F[Resolve Domain matching the Subcommand]
    F --> G[Generate executable command SystemCommand/CompoundCommand]
    G --> H{Print or Dry-run?}
    H -- Yes --> I[Print Command String / Simulate]
    H -- No --> J[Execute underlying command with stdin/args]
    
    %% Interactive execution flow
    B -- Yes [ao interactive] --> K[Detect Host OS & Load Domains]
    K --> L[Select Domain from menu]
    L --> M[Select Subcommand/Action from menu]
    M --> N[Prompt for Positional & Optional parameters with validation]
    N --> O[Generate args_vec dynamically]
    O --> P[Build full app CLI tree & parse args_vec]
    P --> F
    
    J --> Z[End]
    I --> Z
```

---

## 2. Abstraction Model

`ao` separates system operations into logical domains (like `user`, `network`, `package`, `service`) using Rust traits. This separates platform-agnostic CLI definition and parsing from platform-specific execution.

```mermaid
classDiagram
    direction TB
    class Domain {
        <<interface>>
        +name() &str
        +command() clap::Command
        +execute(matches, app) Result~Box~dyn ExecutableCommand~~
        +complete(...) Result~Vec~String~~
    }
    
    class UserManager {
        <<interface>>
        +list(...)
        +add(...)
        +delete(...)
        +modify_user(...)
        +passwd(...)
    }

    class StandardUser {
        +list(...)
        +add(...)
        +delete(...)
        +modify_user(...)
    }

    class MacOSUser {
        +list(...)
        +add(...)
        +delete(...)
        +modify_user(...)
    }

    class SystemCommand {
        +binary String
        +args Vec~String~
        +stdin_data Option~String~
        +execute() Result
        +as_string() String
    }

    class CompoundCommand {
        +commands Vec~Box~dyn ExecutableCommand~~
        +execute() Result
    }

    Domain <|-- StandardUser
    Domain <|-- MacOSUser
    UserManager <|-- StandardUser
    UserManager <|-- MacOSUser
    ExecutableCommand <|-- SystemCommand
    ExecutableCommand <|-- CompoundCommand
    
    StandardUser ..> SystemCommand : Generates/Wraps
    StandardUser ..> CompoundCommand : Generates/Wraps
```

---

## 3. Dashboard Architecture

The interactive system monitor (`ao dashboard`) runs a decoupled event loop built using the `ratatui` crate. It isolates rendering logic from background system polling to ensure interface responsiveness.

```mermaid
flowchart LR
    subgraph UI Event Loop [Main Thread]
        A[Terminal Events / Keypress] --> B[Handle Input / Tab Navigation / Tree Filters]
        B --> C[App State Update]
        D[Tick Event] --> E[Draw Call / Render Widgets]
        C --> E
    end

    subgraph Data Synchronization [Background Cycle]
        F[Stratified Data Poller] --> G[Refresh CPU / Memory / Disks]
        G --> H[RSS Memory computation via /proc/statm]
        H --> C
    end

    E -.-> F
```
