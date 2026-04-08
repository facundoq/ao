# ao

Admin Operations (`ao`) is a centralized, unified command line wrapper written in Rust designed to perform sysop operations across fragmented Linux environments. Instead of context-switching between `apt`, `dnf`, `systemctl`, `usermod`, and `ip`, you just use `ao`.

## Philosophy

* **Ergonomic Speed:** Fast, predictable muscle memory (`ao <domain> <action>`).
* **Zero Overhead:** Instantaneous startup time via Rust.
* **Abstraction:** Hides the differences between Debian, Fedora, Arch, etc.

## Feature Support Matrix

This matrix tracks the current implementation status of the syntax tree defined in `SPEC.md`. Because `ao` abstracts the underlying Operating System, some features may only be implemented for specific backend distributions (e.g., Debian/Ubuntu vs Fedora/RHEL).

| Domain | Sub-domain | Description | Status | Level of Support |
| :--- | :--- | :--- | :--- | :--- |
| `ao sys` | Core System | Updates, power, time | ✅ Implemented | Generic Linux |
| `ao svc` | Services | Start, stop, list services (`systemctl`) | ✅ Implemented | Debian/Ubuntu (Systemd) |
| `ao user` | Users | Add, remove, modify users | ✅ Implemented | Generic Linux |
| `ao group` | Groups | Add, remove, modify groups | ✅ Implemented | Generic Linux |
| `ao net` | Networking | Interfaces, IPs, routes | ✅ Implemented | Generic Linux |
| `ao net fw` | Firewall | Allow, block, status | ✅ Implemented | Generic Linux |
| `ao net wifi` | Wi-Fi | Scan, connect, forget | ✅ Implemented | Generic Linux |
| `ao disk` | Storage | Mount, unmount, usage | ✅ Implemented | Generic Linux |
| `ao pkg` | Packages | Install, remove, update | ✅ Implemented | Debian/Ubuntu (`apt`) |
| `ao log` | Logs | Tail service and system logs | ✅ Implemented | Generic Linux |
| `ao boot` | Boot & Kernel | Bootloader defaults, kernel modules | ✅ Implemented | Generic Linux |
| `ao gui` | Displays | Wayland/X11 detect, resolution | ✅ Implemented | Generic Linux |
| `ao dev` | Devices | PCI/USB lists, Bluetooth, Printers | ✅ Implemented | Generic Linux |
| `ao virt` | Virtualization | Docker, Podman, libvirt abstraction | ✅ Implemented | Generic Linux |
| `ao sec` | Security | SELinux/AppArmor contexts, audits | ✅ Implemented | Generic Linux |
| `ao distro` | Distributions | OS info, major release upgrades | ✅ Implemented | Generic Linux |

## Architecture

`ao` relies on an abstraction layer (`src/os/mod.rs`). The CLI (`src/cli.rs`) routes commands to the `Detector` (`src/os/detector.rs`), which reads `/etc/os-release` and returns a Boxed Trait Object implementing the necessary command logic (e.g., `Apt` or `Systemd` in `src/os/debian.rs`).

## Command Tree

```mermaid
graph LR
    subgraph ao
        ao_root(ao) --> pkg
        ao_root --> svc
        ao_root --> user
        ao_root --> group
        ao_root --> disk
        ao_root --> sys
        ao_root --> log
        ao_root --> distro
        ao_root --> net
        ao_root --> boot
        ao_root --> gui
        ao_root --> dev
        ao_root --> virt
        ao_root --> sec
        ao_root --> completions
        ao_root --> monitor
    end

    subgraph pkg
        pkg_root(pkg) --> pkg_update(update)
        pkg_root --> pkg_install(install)
        pkg_root --> pkg_remove(remove)
        pkg_root --> pkg_search(search)
        pkg_root --> pkg_list(list)
    end

    subgraph svc
        svc_root(svc) --> svc_list(list)
        svc_root --> svc_up(up)
        svc_root --> svc_down(down)
        svc_root --> svc_restart(restart)
        svc_root --> svc_reload(reload)
        svc_root --> svc_status(status)
    end

    subgraph user
        user_root(user) --> user_list(list)
        user_root --> user_add(add)
        user_root --> user_del(del)
        user_root --> user_mod(mod)
        user_root --> user_passwd(passwd)
    end

    subgraph group
        group_root(group) --> group_list(list)
        group_root --> group_add(add)
        group_root --> group_del(del)
        group_root --> group_mod(mod)
    end

    subgraph disk
        disk_root(disk) --> disk_list(list)
        disk_root --> disk_mount(mount)
        disk_root --> disk_unmount(unmount)
        disk_root --> disk_usage(usage)
    end

    subgraph sys
        sys_root(sys) --> sys_info(info)
        sys_root --> sys_power(power)
        sys_root --> sys_time(time)
    end

    subgraph log
        log_root(log) --> log_tail(tail)
        log_root --> log_sys(sys)
        log_root --> log_file(file)
    end

    subgraph distro
        distro_root(distro) --> distro_info(info)
        distro_root --> distro_upgrade(upgrade)
    end

    subgraph net
        net_root(net) --> net_interfaces(interfaces)
        net_root --> net_ips(ips)
        net_root --> net_routes(routes)
        net_root --> fw
        net_root --> wifi
    end

    subgraph fw
        fw_root(fw) --> fw_status(status)
        fw_root --> fw_allow(allow)
        fw_root --> fw_deny(deny)
    end

    subgraph wifi
        wifi_root(wifi) --> wifi_scan(scan)
        wifi_root --> wifi_connect(connect)
    end

    subgraph boot
        boot_root(boot) --> boot_list(list)
        boot_root --> boot_mod(mod)
    end
    
    subgraph boot_mod
        boot_mod_root(mod) --> boot_mod_list(list)
        boot_mod_root --> boot_mod_load(load)
        boot_mod_root --> boot_mod_unload(unload)
    end

    subgraph gui
        gui_root(gui) --> gui_info(info)
        gui_root --> gui_display(display)
    end

    subgraph gui_display
        gui_display_root(display) --> gui_display_list(list)
    end

    subgraph dev
        dev_root(dev) --> dev_list(list)
        dev_root --> dev_pci(pci)
        dev_root --> dev_usb(usb)
        dev_root --> bt
        dev_root --> print
    end
    
    subgraph bt
        bt_root(bt) --> bt_status(status)
        bt_root --> bt_scan(scan)
        bt_root --> bt_pair(pair)
        bt_root --> bt_connect(connect)
    end

    subgraph print
        print_root(print) --> print_list(list)
    end
    
    subgraph virt
        virt_root(virt) --> virt_ps(ps)
        virt_root --> virt_start(start)
        virt_root --> virt_stop(stop)
        virt_root --> virt_rm(rm)
        virt_root --> virt_logs(logs)
    end

    subgraph sec
        sec_root(sec) --> sec_audit(audit)
        sec_root --> sec_context(context)
    end

    subgraph completions
        completions_root(completions) --> completions_generate(generate)
        completions_root --> completions_install(install)
        completions_root --> completions_setup(setup)
    end

    linkStyle 0 stroke-width:2px,fill:none,stroke:red;
    linkStyle 1 stroke-width:2px,fill:none,stroke:orange;
    linkStyle 2 stroke-width:2px,fill:none,stroke:yellow;
    linkStyle 3 stroke-width:2px,fill:none,stroke:green;
    linkStyle 4 stroke-width:2px,fill:none,stroke:blue;
    linkStyle 5 stroke-width:2px,fill:none,stroke:indigo;
    linkStyle 6 stroke-width:2px,fill:none,stroke:violet;
    linkStyle 7 stroke-width:2px,fill:none,stroke:red;
    linkStyle 8 stroke-width:2px,fill:none,stroke:orange;
    linkStyle 9 stroke-width:2px,fill:none,stroke:yellow;
    linkStyle 10 stroke-width:2px,fill:none,stroke:green;
    linkStyle 11 stroke-width:2px,fill:none,stroke:blue;
    linkStyle 12 stroke-width:2px,fill:none,stroke:indigo;
    linkStyle 13 stroke-width:2px,fill:none,stroke:violet;
    linkStyle 14 stroke-width:2px,fill:none,stroke:red;
    linkStyle 15 stroke-width:2px,fill:none,stroke:orange;
```
