# `ao dashboard` - Ratatui TUI Design

The `ao dashboard` command will launch a rich, interactive Terminal User Interface (TUI) using the `ratatui` crate. It will act as a unified system monitor and administration hub, akin to `htop` or `btop`, but encompassing all the domains managed by `ao`.

## Overall Layout

The UI will be structured with a fixed header, a main content area (tab-based), and a fixed footer.

*   **Header:**
    *   Left: Application title (`ao dashboard`) and current version.
    *   Center: System Hostname and Uptime.
    *   Right: Current time and user.
*   **Main Content Area (Tabs):**
    *   A set of navigable tabs at the top of the main area (using left/right arrows or numbers to switch).
    *   The content area will render data specific to the active tab.
*   **Footer:**
    *   Context-sensitive help and keybindings (e.g., `[q] Quit`, `[1-6] Switch Tabs`, `[Enter] Details`).

## Tab Structure & Content

### 1. Overview (Default Tab)
A high-level summary of system health.
*   **Top Half:** Gauges/Bar charts for CPU, RAM, and Swap usage.
*   **Bottom Left:** Disk usage summary (top mounted partitions).
*   **Bottom Right:** Quick network stats (Rx/Tx rates).

### 2. Processes (The "htop" view)
Detailed view of running processes.
*   **Table:** PID, User, Priority, CPU %, MEM %, Command.
*   **Interactivity:** Sortable columns (by CPU, MEM, etc.). Ability to select a process to send signals (kill, stop).

### 3. Users & Sessions
Live view of system users and active sessions.
*   **Left Pane:** List of active sessions (similar to `ao user session`).
*   **Right Pane:** System users list or details about the currently selected user/session.

### 4. Network & Firewall
Network interfaces, routing, and security.
*   **Top Split:** Network interfaces (IPs, MAC, State).
*   **Bottom Split:** Active firewall rules or live connections (using something similar to `ss` or `netstat` data).

### 5. Services (Systemd)
Management of system services.
*   **List:** Service name, Status (active, failed, exited), Description.
*   **Interactivity:** Select a service to Restart, Stop, or view the last few log lines.

### 6. Virtualization (VMs/Containers)
Overview of Docker/Podman containers and VMs.
*   **List:** Name/ID, Image, Status, CPU/Mem usage (if available).
*   **Interactivity:** Start, Stop, or View Logs of the selected container/VM.

## Interactivity & Keybindings

*   **Global Bindings:**
    *   `q` or `Esc`: Quit the dashboard.
    *   `Tab` / `Shift+Tab`: Cycle through tabs (or `1` through `6`).
*   **Tab-Specific Bindings:**
    *   `Up` / `Down` or `j` / `k`: Scroll through lists/tables.
    *   `Enter`: View details of the selected item.
    *   `/`: Search/filter the current list (e.g., search processes by name).
    *   `F10` or `x`: Perform context action (e.g., kill process, restart service).

## Implementation Strategy

1.  **Dependencies:** Add `ratatui` and `crossterm` to `Cargo.toml`.
2.  **App State:** Create an `App` struct to hold the UI state (current tab, selected index, loaded data).
3.  **Data Fetching:** Implement asynchronous or background data fetching so the UI thread doesn't block while gathering system stats (e.g., using `tokio` or standard threading).
4.  **UI Rendering:** Modularize rendering logic (e.g., `draw_tabs`, `draw_overview`, `draw_processes`) to keep the main loop clean.
5.  **Integration:** Add the `Dashboard` variant to `CliCommand` and execute the TUI loop when `ao dashboard` is called.

This dashboard will leverage the existing abstractions in `src/os/mod.rs` (like `SysManager`, `ServiceManager`, `UserManager`) to pull data, ensuring it remains cross-distribution compatible.