# MacOS Support Plan

## Overview
Implement a MacOS backend for the `ao` CLI to provide cross-platform system administration capabilities.

## Technical Strategy
- **Abstraction:** Leverage the existing domain-based architecture in `src/os/` to implement MacOS-specific modules.
- **Docker Testing:** Use `Docker-OSX` to emulate a MacOS environment for automated testing and CI.
- **Backend:** Implement MacOS-specific versions of `sysinfo`, network management (`networksetup`), user management (`dscl`), and system management (`launchctl`).

## Phases

### Phase 1: Environment Setup
- Configure `Docker-OSX` container with a basic macOS image.
- Set up automated CI integration for macOS-compatible cross-compilation/testing.

### Phase 2: Domain Implementation
- Implement `macos.rs` backend for each domain.
- Map system commands (e.g., replace `systemctl` with `launchctl`).
- Adjust `detector.rs` to identify MacOS.

### Phase 3: Testing & Iteration
- Run current linux tests on MacOS backend.
- Identify and fix incompatibilities.
- Iterate until `cargo test` passes.
