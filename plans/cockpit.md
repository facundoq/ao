# Implementation Plan: Bridging the Gap with Cockpit

Cockpit provides a powerful web-based interface for Linux servers. To reach parity and provide a CLI-first alternative, `ao` should implement the following advanced features.

## 1. Advanced Storage Management (`ao disk adv`)
Currently, `ao disk` handles basic listing and mounting. We need to abstract LVM and RAID.

### Features:
- `ao disk lvm list`: List Volume Groups (VGs) and Logical Volumes (LVs).
- `ao disk lvm create <vg> <name> <size>`: Create new logical volumes.
- `ao disk raid status`: Show health of MDADM arrays.
- `ao disk raid create`: Guide the user through creating a RAID array.

### Tools to Wrap:
- `lvm2` (vgs, lvs, lvcreate)
- `mdadm`

## 2. Advanced Networking (`ao net adv`)
Abstracting complex virtual networking beyond simple IP assignment.

### Features:
- `ao net bridge create <name> <iface...>`: Create a network bridge.
- `ao net bond create <name> <mode> <iface...>`: Create a bonded interface.
- `ao net vlan create <id> <parent>`: Create a tagged VLAN interface.

### Tools to Wrap:
- `nmcli` (Preferred for persistence)
- `ip link` (Fallback)

## 3. SELinux/AppArmor Deep Integration (`ao sec adv`)
Moving beyond `ao sec context` to active management.

### Features:
- `ao sec denials`: Parse audit logs for recent security denials and suggest fixes (like `audit2allow`).
- `ao sec boolean <name> <on/off>`: Toggle SELinux booleans.
- `ao sec mode <enforcing/permissive/disabled>`: Change active enforcement mode permanently.

### Tools to Wrap:
- `semanage`, `setsebool`, `ausearch`

## 4. Performance History (`ao monitor history`)
`ao monitor` is currently live-only.

### Features:
- `ao monitor history --last 24h`: Show average CPU/RAM/IO usage over a period.
- Requires a data collection backend.

### Implementation Strategy:
- Integrate with `Performance Co-Pilot (PCP)` or `sar` (sysstat).
- Parse existing system archives if available.

## 5. Diagnostic Reporting (`ao sys diag`)
Helping users gather information for support.

### Features:
- `ao sys diag report`: Generate a comprehensive diagnostic tarball.
- `ao sys diag check`: Run a suite of health checks (Disk health, swap pressure, failed units).

### Tools to Wrap:
- `sosreport` or `redhat-support-tool`
- `smartctl` (for disk health)

## 6. Software Lifecycle (`ao pkg adv`)
- `ao pkg history`: Show a timeline of what was installed and when.
- `ao pkg rollback`: Revert the last package operation (if supported by backend like DNF).
- `ao pkg autoupdate <enable/disable>`: Manage unattended upgrades.
