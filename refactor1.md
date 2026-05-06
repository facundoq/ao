# Refactor Plan: String Smells & Duplication (refactor1.md)

This document identifies instances of hardcoded strings and code duplication that violate DRY (Don't Repeat Yourself) principles and increase the risk of regressions during refactoring.

## 1. Hardcoded Executable Name ("ao")
**Issue:** The string `"ao"` is hardcoded in every single `is_completing_arg` call across all domain implementations.
**Impact:** If the project is renamed or the binary name changes (e.g., to `ao-cli`), all completion logic will break.
**Fix:** Define a constant in `src/main.rs` or `src/lib.rs`.
```rust
pub const EXECUTABLE_NAME: &str = "ao";
```

## 2. Hardcoded Domain Names
**Issue:** Domain names like `"package"`, `"user"`, `"service"` are hardcoded in `complete` methods.
**Impact:** Duplication of the value returned by `Domain::name()`. If a domain is renamed (as done recently from `pkg` to `package`), many strings must be updated manually.
**Fix:** Use `self.name()` within the `complete` method.

## 3. Hardcoded Action Names
**Issue:** Subcommand actions like `"add"`, `"delete"`, `"modify"` are hardcoded strings.
**Impact:** These strings are already defined in the `CliCommand` and action enums (e.g., `PackageAction`). 
**Fix:** 
- Use the `strum` crate to derive string representations from enum variants automatically.
- Or, define constants within the Domain implementation if they are used multiple times.

## 4. Repetitive `is_completing_arg` Structure
**Issue:** Almost every `complete` method starts with a series of `if is_completing_arg(words, &["ao", "domain", "action"], ...)` blocks.
**Fix:** Create a helper method or a macro that automatically prepends `EXECUTABLE_NAME` and `self.name()` to the check.
```rust
// Proposed helper
fn is_completing_action(&self, words: &[&str], action: &str, pos: usize) -> bool {
    is_completing_arg(words, &[EXECUTABLE_NAME, self.name(), action], pos, false)
}
```

## 5. Duplication in `detect_system`
**Issue:** The `DetectedSystem` struct is instantiated 4 times with almost identical box allocations.
**Fix:** Use a builder pattern or a default instantiation that is subsequently modified based on the detected distro.

## 6. Hardcoded Paths and Magic Strings in `detector.rs`
**Issue:** Strings like `"/etc/os-release"`, `"ID=ubuntu"`, `"ID=debian"` are used directly.
**Fix:** Use constants for system paths and distro identifiers.

## 7. Command Strings in `SystemCommand`
**Issue:** Many `SystemCommand::new("...")` calls use hardcoded strings for common tools like `lsblk`, `docker`, `systemctl`.
**Fix:** Use constants for external tool names to centralize their definitions (and potentially handle cases where paths differ).

## 8. Duplication in Package Managers (`Apt`, `Apk`, `Pacman`, `Dnf`)
**Issue:** The `Domain` implementation for all package managers is nearly identical. Each one manually handles the same `execute` logic (matching `PackageAction`) and `complete` logic.
**Impact:** High maintenance overhead. Adding a new package manager or a new package action requires updating 4+ files with the exact same boilerplate.
**Fix:** Implement a generic `PackageDomain<T: PackageManager>` that handles the `Domain` trait methods once, delegating to the specific `PackageManager` implementation for the actual tool-specific commands.

```rust
pub struct PackageDomain<T: PackageManager> {
    pub manager: T,
}

impl<T: PackageManager> Domain for PackageDomain<T> {
    fn name(&self) -> &'static str { "package" }
    // ... unified execute and complete logic ...
}
```

---

### Priority Implementation
The highest priority is fixing the `complete` methods as they are the most frequent offenders of string duplication.
