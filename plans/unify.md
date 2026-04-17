# Unification Plan: Verbs and Internals

This plan outlines the steps to unify the command verbs across the `ao` CLI and its internal implementation to reduce cognitive load and improve consistency.

## Verb Mapping

| Old Verb | New Verb | Context |
|----------|----------|---------|
| `install` | `add` | Packages, completions, etc. |
| `uninstall` / `remove` / `rm` | `del` | Packages, users, groups, containers, etc. |
| `list` / `ps` | `ls` | All domains |

## Proposed Replacements (Lowering Cognitive Burden)

| Domain | Action | New Command | Note |
|--------|--------|-------------|------|
| `pkg` | `install` | `ao pkg add` | |
| `pkg` | `remove` | `ao pkg del` | |
| `pkg` | `list` | `ao pkg ls` | |
| `virt` | `ps` | `ao virt ls` | Unified with other lists |
| `virt` | `rm` | `ao virt del` | Unified with other deletions |
| `svc` | `list` | `ao svc ls` | |
| `user` | `list` | `ao user ls` | |
| `boot` | `list` | `ao boot ls` | |
| `boot mod`| `list` | `ao boot mod ls` | |
| `dev` | `list` | `ao dev ls` | |
| `dev print`| `list` | `ao dev print ls` | |

## Task List

### 1. CLI Definition Updates (`src/cli.rs`)
- [x] Rename `PkgAction::Install` to `Add`
- [x] Rename `PkgAction::Remove` to `Del`
- [x] Rename `PkgAction::List` to `Ls`
- [x] Rename `SvcAction::List` to `Ls`
- [x] Rename `UserAction::List` to `Ls`
- [x] Rename `GroupAction::List` to `Ls`
- [x] Rename `DiskAction::List` to `Ls`
- [x] Rename `BootAction::List` to `Ls`
- [x] Rename `BootModAction::List` to `Ls`
- [x] Rename `GuiDisplayAction::List` to `Ls`
- [x] Rename `DevAction::List` to `Ls`
- [x] Rename `PrintAction::List` to `Ls`
- [x] Rename `VirtAction::Ps` to `Ls`
- [x] Rename `VirtAction::Rm` to `Del`

### 2. Internal Trait & Method Updates (`src/os/mod.rs`)
- [x] Rename `PackageManager::install` to `add`
- [x] Rename `PackageManager::remove` to `del`
- [x] Rename `PackageManager::list` to `ls`
- [x] Rename `ServiceManager::list` to `ls`
- [x] Rename `UserManager::list` to `ls`
- [x] Rename `GroupManager::list` to `ls`
- [x] Rename `DiskManager::list` to `ls`
- [x] Rename `BootManager::list_entries` to `ls_entries`
- [x] Rename `BootManager::list_modules` to `ls_modules`
- [x] Rename `GuiManager::list_displays` to `ls_displays`
- [x] Rename `DevManager::list` to `ls`
- [x] Rename `DevManager::list_printers` to `ls_printers`
- [x] Rename `VirtManager::ps` to `ls`
- [x] Rename `VirtManager::rm` to `del`

### 3. Domain Implementation Updates
- [x] Update `debian.rs`, `fedora.rs`, `arch.rs`
- [x] Update `linux_generic/*.rs`

### 4. New "Self" Domain
- [x] Create `src/os/linux_generic/self_domain.rs`
- [x] Move `CompletionsAction` into `SelfAction::Completions`
- [x] Add `SelfAction::Info`
- [x] Add `SelfAction::Update` (placeholder)
- [x] Register `self` domain in `detector.rs`
