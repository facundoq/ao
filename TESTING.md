# Testing Strategy for `ao`

Testing a system administration tool like `ao` presents unique challenges. Because `ao` directly interacts with low-level Linux subsystems—such as `systemd` for services, `iproute2` for networking, and kernel structures for disk management—standard Docker containers are insufficient. Containers isolate processes but share the host kernel, and typically lack a true `init` system (like `systemd`) or the privileges necessary to mutate system state safely.

To ensure `ao` is robust, reliable, and destructive only when intended, our testing strategy relies on **Virtual Machines (VMs)** and **External Orchestration**.

## 1. Environment: Virtual Machines (VMs)

All integration and end-to-end (E2E) tests must run inside fully virtualized environments. This guarantees a complete OS stack with an isolated kernel, real `init` systems, and safe sandbox environments for destructive commands (e.g., adding/removing users, modifying network routes, or partitioning virtual disks).

**Recommended Tooling:**
* **Vagrant** (with VirtualBox or libvirt providers) or **Multipass** (for lightweight Ubuntu VMs). Vagrant is preferred for its rich ecosystem of multi-distribution boxes.

## 2. Multi-Distribution Support

`ao` abstracts system administration across different Linux flavors (e.g., `ao pkg install` mapping to `apt`, `dnf`, or `pacman`). To verify these abstractions, our VM matrix must include multiple distributions:

* **Debian / Ubuntu:** To test `apt` package management and Debian-specific service configurations.
* **Fedora / CentOS / RHEL:** To test `dnf` / `yum` and Red Hat ecosystem nuances.
* **Arch Linux (Optional but recommended):** To test `pacman` and bleeding-edge system configurations.

The testing pipeline should be capable of spinning up all target distributions simultaneously to run the test suite against each.

## 3. Test Orchestration (External)

Because we are testing system-level changes, running `cargo test` directly inside the VM is not the most effective approach for E2E tests. Instead, tests will be orchestrated **externally** from the host machine.

### The Testing Flow:

1. **Compilation:** The `ao` Rust binary is compiled on the host machine for the target architecture (e.g., `x86_64-unknown-linux-musl` for maximum portability without glibc mismatch issues).
2. **Provisioning:** The VM hypervisor spins up the target VMs (e.g., `vagrant up debian fedora`).
3. **Deployment:** The compiled `ao` binary is securely copied into each VM (e.g., via SSH/SCP to `/usr/local/bin/ao`).
4. **Execution:** An external testing framework runs the tests by executing commands via SSH and asserting the standard output, standard error, and exit codes.

### External Framework: Bats (Bash Automated Testing System)

[Bats](https://github.com/bats-core/bats-core) is highly recommended for this external orchestration. It allows us to write simple scripts that send commands to the VM and assert the results.

**Example Bats Test (`test_pkg.bats`):**

```bash
#!/usr/bin/env bats

setup() {
  # Define the target VM and SSH options
  export VM_SSH="vagrant ssh debian -c"
}

@test "ao pkg install successfully installs curl" {
  # 1. Ensure curl is not installed
  $VM_SSH "sudo apt-get remove -y curl"

  # 2. Run ao command
  run $VM_SSH "sudo ao pkg install curl"

  # 3. Assert success
  [ "$status" -eq 0 ]

  # 4. Verify curl is now executable
  run $VM_SSH "which curl"
  [ "$status" -eq 0 ]
}

@test "ao user add creates a new user" {
  # 1. Run ao user add
  run $VM_SSH "sudo ao user add testuser"
  [ "$status" -eq 0 ]

  # 2. Verify user exists in /etc/passwd
  run $VM_SSH "grep testuser /etc/passwd"
  [ "$status" -eq 0 ]

  # Cleanup
  $VM_SSH "sudo userdel -r testuser"
}
```

## 4. CI/CD Integration

In a Continuous Integration environment (like GitHub Actions):
1. Runner executes `cargo build --release`.
2. Runner spins up headless VMs using Vagrant/VirtualBox or QEMU/KVM.
3. The external test suite (Bats) is executed against the local VMs.
4. If all tests pass, the PR is marked as green.

## 5. Unit Testing

While E2E tests require VMs, internal Rust unit tests should still be written for logic that does not touch the system.
* Parsing logic, string formatting, and OS-detection heuristics can be tested normally using `cargo test`.
* Code that modifies the system should be hidden behind traits/interfaces so that a "mock" system can be injected for fast unit testing where possible.
