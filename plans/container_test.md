# Multi-Distro Container Testing Plan

To ensure **ao**'s portability and correct behavior across different Linux environments, we will use **Incus** (or LXC) to run automated integration tests on various distributions.

## 1. Environment Setup
The testing host must have `incus` installed and configured.

### Required Images:
- `images:debian/12`
- `images:ubuntu/24.04`
- `images:archlinux`
- `images:fedora/40`
- `images:alpine/3.20`

## 2. Test Execution Script (`scripts/test-distros.sh`)
We will create a bash script that iterates through the distributions and performs the following for each:

1. **Provisioning**: Create a new container if it doesn't exist (`incus launch <image> <name>`).
2. **Binary Transfer**: Copy the latest statically compiled `ao` binary (built with `musl`) into the container.
3. **Execution**: Run the `read_only_tests.sh` suite inside the container.
4. **Cleanup**: Stop and delete the container (optional, or reuse for speed).

### Sample Workflow Logic:
```bash
for DISTRO in debian ubuntu arch fedora alpine; do
  echo "--- Testing on $DISTRO ---"
  CONTAINER="ao-test-$DISTRO"
  
  # Launch if needed
  incus start "$CONTAINER" || incus launch "images:$DISTRO" "$CONTAINER"
  
  # Wait for network
  sleep 2
  
  # Push binary and test script
  incus file push ./target/x86_64-unknown-linux-musl/release/ao "$CONTAINER/usr/local/bin/ao"
  incus file push ./tests/read_only_tests.sh "$CONTAINER/tmp/test.sh"
  
  # Run tests
  incus exec "$CONTAINER" -- bash /tmp/test.sh
done
```

## 3. Specific Distro Considerations
- **Alpine**: Uses `musl` natively; verify that our static binary behaves identically to glibc-based distros.
- **Arch/Fedora**: Verify that domain-specific tools (like `pacman` or `dnf`) are correctly detected by `ao`.
- **Systemd vs OpenRC**: Test how `ao svc` handles different init systems (Alpine typically uses OpenRC, others use Systemd).

## 4. Automation Strategy
- Integrate this script into the local development workflow before any major release.
- (Optional) Use a dedicated runner for CI if hardware acceleration (KVM/LXC) is available.
