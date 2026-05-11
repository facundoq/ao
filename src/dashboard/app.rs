use crate::config::Config;
use crate::dashboard::rss::{get_process_memory, get_rss};
use crate::os::detector::DetectedSystem;
use crate::os::{ContainerInfo, NetInterfaceInfo, SensorInfo, ServiceInfo, UserSessionInfo};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use sysinfo::{
    CpuRefreshKind, Disks, MemoryRefreshKind, Networks, Process, RefreshKind, System, Users,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSort {
    Pid,
    Cpu,
    Mem,
    Name,
    User,
}

pub struct App<'a> {
    pub system_info: System,
    pub disks: Disks,
    pub networks: Networks,
    pub users_list: Users,
    pub detected_system: &'a DetectedSystem,
    pub tab_index: usize,
    pub tabs: Vec<&'static str>,
    pub selected_index: usize,

    pub sessions: Vec<UserSessionInfo>,
    pub users: Vec<(String, bool)>,
    pub interfaces: Vec<NetInterfaceInfo>,
    pub services: Vec<ServiceInfo>,
    pub containers: Vec<ContainerInfo>,
    pub sensors: Vec<SensorInfo>,
    pub ram_config: String,
    pub ram_speed: String,

    pub cpu_history: Vec<(f64, f64)>,
    pub mem_history: Vec<(f64, f64)>,
    pub swap_history: Vec<(f64, f64)>,
    pub net_rx_history: Vec<(f64, f64)>,
    pub net_tx_history: Vec<(f64, f64)>,
    pub history_start: Instant,

    pub last_tick_time: Instant,
    pub prev_network_data: HashMap<String, (u64, u64)>,
    pub network_speeds: HashMap<String, (u64, u64)>,

    pub process_sort: ProcessSort,
    pub sort_descending: bool,
    pub use_tree_view: bool,
    pub tree_expansion_depth: u32,
    pub show_only_current_user: bool,
    pub hide_kernel_processes: bool,
    pub total_rss: u64,
    pub process_tree: Vec<ProcessTreeNode>,
    pub top_cpu_processes: Vec<ProcessSummary>,
    pub top_mem_processes: Vec<ProcessSummary>,
    pub sorted_processes: Vec<ProcessSummary>,
    pub flattened_tree: Vec<FlattenedTreeNode>,
    pub last_process_refresh: Instant,
    pub refresh_interval: Duration,
    pub max_temps: HashMap<String, f32>,
    pub process_filter: String,
    pub is_filtering: bool,
}

#[derive(Debug, Clone)]
pub struct ProcessTreeNode {
    pub pid: sysinfo::Pid,
    pub name: String,
    pub user: String,
    pub cpu_usage: f32,
    pub memory: u64,
    pub virt_mem: u64,
    pub shared_mem: u64,
    pub depth: u32,
    pub children: Vec<ProcessTreeNode>,
    pub total_cpu: f32,
    pub total_mem: u64,
    pub total_virt: u64,
    pub total_shared: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessSummary {
    pub pid: sysinfo::Pid,
    pub name: String,
    pub user: String,
    pub memory: u64,
    pub virt_mem: u64,
    pub shared_mem: u64,
    pub cpu: f32,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct FlattenedTreeNode {
    pub pid: String,
    pub user: String,
    pub cpu: String,
    pub mem: String,
    pub virt: String,
    pub shared: String,
    pub name: String,
    pub depth: u32,
}

impl<'a> App<'a> {
    pub fn new(detected_system: &'a DetectedSystem) -> Self {
        let config = Config::load().unwrap_or_default();
        let mut system_info = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(sysinfo::ProcessRefreshKind::everything()),
        );
        system_info.refresh_all();

        let networks = Networks::new_with_refreshed_list();
        let mut prev_network_data = HashMap::new();
        for (name, network) in &networks {
            prev_network_data.insert(
                name.clone(),
                (network.total_received(), network.total_transmitted()),
            );
        }

        let sys_info = detected_system.sys.get_info().ok();
        let ram_config = sys_info
            .as_ref()
            .map(|s| s.ram_config.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        let ram_speed = sys_info
            .as_ref()
            .map(|s| s.ram_speed.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let mut app = Self {
            system_info,
            disks: Disks::new_with_refreshed_list(),
            networks,
            users_list: Users::new_with_refreshed_list(),
            detected_system,
            tab_index: 0,
            tabs: vec![
                "Overview",
                "Storage",
                "Process",
                "User",
                "Network",
                "Service",
                "Virtualization",
                "Sensors",
                "Charts",
            ],
            selected_index: 0,
            sessions: Vec::new(),
            users: Vec::new(),
            interfaces: Vec::new(),
            services: Vec::new(),
            containers: Vec::new(),
            sensors: Vec::new(),
            ram_config,
            ram_speed,
            cpu_history: Vec::with_capacity(61),
            mem_history: Vec::with_capacity(61),
            swap_history: Vec::with_capacity(61),
            net_rx_history: Vec::with_capacity(61),
            net_tx_history: Vec::with_capacity(61),
            history_start: Instant::now(),
            last_tick_time: Instant::now(),
            prev_network_data,
            network_speeds: HashMap::new(),
            process_sort: ProcessSort::Cpu,
            sort_descending: true,
            use_tree_view: true,
            tree_expansion_depth: 2,
            show_only_current_user: false,
            hide_kernel_processes: !config.ui.show_kernel_processes,
            total_rss: 0,
            process_tree: Vec::new(),
            top_cpu_processes: Vec::new(),
            top_mem_processes: Vec::new(),
            sorted_processes: Vec::new(),
            flattened_tree: Vec::new(),
            last_process_refresh: Instant::now() - Duration::from_secs(11),
            refresh_interval: Duration::from_secs(10),
            max_temps: HashMap::new(),
            process_filter: String::new(),
            is_filtering: false,
        };
        app.refresh_users_list();
        app.on_tick();
        app
    }

    pub fn refresh_users_list(&mut self) {
        let mut users = Vec::new();
        for user in &self.users_list {
            let uid_str = user.id().to_string();
            let uid = uid_str.parse::<u32>().unwrap_or(0);
            users.push((user.name().to_string(), uid < 1000));
        }
        users.sort_by(|a, b| match (a.1, b.1) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => a.0.cmp(&b.0),
        });
        self.users = users;
    }

    pub fn on_tick(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick_time).as_secs_f64();
        self.last_tick_time = now;

        self.system_info.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            sysinfo::ProcessRefreshKind::everything(),
        );
        self.system_info.refresh_memory();
        self.system_info.refresh_cpu_usage();
        self.disks.refresh(true);
        self.networks.refresh(true);
        self.refresh_process_data(false);

        let now_sec = now.duration_since(self.history_start).as_secs_f64();
        self.cpu_history
            .push((now_sec, self.system_info.global_cpu_usage() as f64));
        if self.cpu_history.len() > 60 {
            self.cpu_history.remove(0);
        }

        let mem_total = self.system_info.total_memory() as f64;
        let mem_percent = if mem_total > 0.0 {
            (self.system_info.used_memory() as f64 / mem_total) * 100.0
        } else {
            0.0
        };
        self.mem_history.push((now_sec, mem_percent));
        if self.mem_history.len() > 60 {
            self.mem_history.remove(0);
        }

        let swap_total = self.system_info.total_swap() as f64;
        let swap_percent = if swap_total > 0.0 {
            (self.system_info.used_swap() as f64 / swap_total) * 100.0
        } else {
            0.0
        };
        self.swap_history.push((now_sec, swap_percent));
        if self.swap_history.len() > 60 {
            self.swap_history.remove(0);
        }

        let mut total_rx_speed = 0.0;
        let mut total_tx_speed = 0.0;
        for (name, network) in &self.networks {
            let rx = network.total_received();
            let tx = network.total_transmitted();
            if let Some((old_rx, old_tx)) = self.prev_network_data.get(name) {
                let rx_delta = rx.saturating_sub(*old_rx);
                let tx_delta = tx.saturating_sub(*old_tx);
                let rx_speed = (rx_delta as f64 / elapsed) as u64;
                let tx_speed = (tx_delta as f64 / elapsed) as u64;
                self.network_speeds
                    .insert(name.clone(), (rx_speed, tx_speed));
                total_rx_speed += rx_speed as f64;
                total_tx_speed += tx_speed as f64;
            }
            self.prev_network_data.insert(name.clone(), (rx, tx));
        }
        self.net_rx_history.push((now_sec, total_rx_speed));
        if self.net_rx_history.len() > 60 {
            self.net_rx_history.remove(0);
        }
        self.net_tx_history.push((now_sec, total_tx_speed));
        if self.net_tx_history.len() > 60 {
            self.net_tx_history.remove(0);
        }

        if let Ok(s) = self.detected_system.overview.get_sensors() {
            for sensor in &s {
                let entry = self
                    .max_temps
                    .entry(sensor.label.clone())
                    .or_insert(sensor.temperature);
                if sensor.temperature > *entry {
                    *entry = sensor.temperature;
                }
            }
            self.sensors = s;
        }

        if let Ok(mut s) = self.detected_system.user.get_sessions(None, true, Some(50)) {
            s.reverse();
            self.sessions = s;
        }
        if let Ok(i) = self.detected_system.net.get_interfaces() {
            self.interfaces = i;
        }
        if let Ok(s) = self.detected_system.svc.get_all_services_info() {
            self.services = s;
        }
        if let Ok(c) = self.detected_system.virt.get_containers() {
            self.containers = c;
        }
    }

    pub fn refresh_process_data(&mut self, force: bool) {
        if !force && self.last_process_refresh.elapsed() < self.refresh_interval {
            return;
        }
        self.last_process_refresh = Instant::now();
        self.total_rss = self
            .system_info
            .processes()
            .keys()
            .map(|pid| get_rss(*pid))
            .sum();
        self.process_tree = self.calculate_process_tree();

        let all_summaries = self.get_sorted_processes(); // Already filtered

        let mut top_cpu = all_summaries.clone();
        top_cpu.sort_by(|a, b| {
            b.cpu
                .partial_cmp(&a.cpu)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.top_cpu_processes = top_cpu.into_iter().take(10).collect();

        let mut top_mem = all_summaries;
        top_mem.sort_by_key(|p| std::cmp::Reverse(p.memory));
        self.top_mem_processes = top_mem.into_iter().take(10).collect();

        self.sorted_processes = self.get_sorted_processes();
        self.flattened_tree = self.calculate_flattened_tree();
    }

    pub fn get_top_cpu_processes(&self, count: usize) -> Vec<ProcessSummary> {
        let mut all = Vec::new();
        fn flatten(node: &ProcessTreeNode, out: &mut Vec<ProcessSummary>) {
            out.push(ProcessSummary {
                pid: node.pid,
                name: node.name.clone(),
                user: node.user.clone(),
                memory: node.total_mem,
                virt_mem: node.total_virt,
                shared_mem: node.total_shared,
                cpu: node.total_cpu,
                command: "".to_string(),
            });
            for child in &node.children {
                flatten(child, out);
            }
        }
        for root in &self.process_tree {
            flatten(root, &mut all);
        }
        all.sort_by(|a, b| {
            b.cpu
                .partial_cmp(&a.cpu)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all.into_iter().take(count).collect()
    }

    pub fn get_top_mem_processes(&self, count: usize) -> Vec<ProcessSummary> {
        let mut all = Vec::new();
        fn flatten(node: &ProcessTreeNode, out: &mut Vec<ProcessSummary>) {
            out.push(ProcessSummary {
                pid: node.pid,
                name: node.name.clone(),
                user: node.user.clone(),
                memory: node.total_mem,
                virt_mem: node.total_virt,
                shared_mem: node.total_shared,
                cpu: node.total_cpu,
                command: "".to_string(),
            });
            for child in &node.children {
                flatten(child, out);
            }
        }
        for root in &self.process_tree {
            flatten(root, &mut all);
        }
        all.sort_by_key(|n| std::cmp::Reverse(n.memory));
        all.into_iter().take(count).collect()
    }

    pub fn get_sorted_processes(&self) -> Vec<ProcessSummary> {
        let current_uid = unsafe { libc::getuid() };
        let mut processes: Vec<&Process> = self
            .system_info
            .processes()
            .values()
            .filter(|p| {
                if self.show_only_current_user {
                    if let Some(uid) = p.user_id() {
                        let uid_u32: u32 = uid.to_string().parse().unwrap_or(u32::MAX);
                        if uid_u32 != current_uid {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                if self.hide_kernel_processes {
                    let pid = p.pid().as_u32();
                    if pid == 0 || pid == 2 {
                        return false;
                    }
                    if let Some(ppid) = p.parent()
                        && ppid.as_u32() == 2
                    {
                        return false;
                    }
                }
                if !self.process_filter.is_empty() {
                    let filter = self.process_filter.to_lowercase();
                    let name = p.name().to_string_lossy().to_lowercase();
                    let cmd = p
                        .exe()
                        .map(|e| e.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    if !name.contains(&filter) && !cmd.contains(&filter) {
                        return false;
                    }
                }
                true
            })
            .collect();
        processes.sort_by(|a, b| {
            let res = match self.process_sort {
                ProcessSort::Pid => a.pid().cmp(&b.pid()),
                ProcessSort::Cpu => a
                    .cpu_usage()
                    .partial_cmp(&b.cpu_usage())
                    .unwrap_or(std::cmp::Ordering::Equal),
                ProcessSort::Mem => get_rss(a.pid()).cmp(&get_rss(b.pid())),
                ProcessSort::Name => a.name().cmp(b.name()),
                ProcessSort::User => {
                    let au = a
                        .user_id()
                        .and_then(|id| self.users_list.get_user_by_id(id))
                        .map(|u| u.name())
                        .unwrap_or("");
                    let bu = b
                        .user_id()
                        .and_then(|id| self.users_list.get_user_by_id(id))
                        .map(|u| u.name())
                        .unwrap_or("");
                    au.cmp(bu)
                }
            };
            if self.sort_descending {
                res.reverse()
            } else {
                res
            }
        });
        processes
            .into_iter()
            .map(|p| {
                let user_name = p
                    .user_id()
                    .and_then(|uid| self.users_list.get_user_by_id(uid))
                    .map(|u| u.name().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let mem = get_process_memory(p.pid());
                ProcessSummary {
                    pid: p.pid(),
                    name: p.name().to_string_lossy().to_string(),
                    user: user_name,
                    memory: mem.rss,
                    virt_mem: mem.virt,
                    shared_mem: mem.shared,
                    cpu: p.cpu_usage() / self.system_info.cpus().len() as f32,
                    command: p
                        .exe()
                        .map(|e| e.to_string_lossy().to_string())
                        .unwrap_or_else(|| "".to_string()),
                }
            })
            .collect()
    }

    pub fn calculate_process_tree(&mut self) -> Vec<ProcessTreeNode> {
        let processes = self.system_info.processes();
        let cpu_count = self.system_info.cpus().len() as f32;
        let mut filtered = HashMap::new();
        let current_uid = unsafe { libc::getuid() };
        for (pid, process) in processes {
            if self.show_only_current_user {
                if let Some(uid) = process.user_id() {
                    let uid_u32: u32 = uid.to_string().parse().unwrap_or(u32::MAX);
                    if uid_u32 != current_uid {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            if self.hide_kernel_processes {
                let pid_u32 = pid.as_u32();
                if pid_u32 == 0 || pid_u32 == 2 {
                    continue;
                }
                if let Some(ppid) = process.parent()
                    && ppid.as_u32() == 2
                {
                    continue;
                }
            }
            if !self.process_filter.is_empty() {
                let filter = self.process_filter.to_lowercase();
                let name = process.name().to_string_lossy().to_lowercase();
                let cmd = process
                    .exe()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                if !name.contains(&filter) && !cmd.contains(&filter) {
                    continue;
                }
            }
            filtered.insert(*pid, process);
        }
        let mut tree: HashMap<sysinfo::Pid, Vec<sysinfo::Pid>> = HashMap::new();
        let mut root_pids = Vec::new();
        for (pid, process) in &filtered {
            if let Some(parent_pid) = process.parent() {
                if filtered.contains_key(&parent_pid) {
                    tree.entry(parent_pid).or_default().push(*pid);
                } else {
                    root_pids.push(*pid);
                }
            } else {
                root_pids.push(*pid);
            }
        }
        fn build_node(
            pid: sysinfo::Pid,
            depth: u32,
            processes: &HashMap<sysinfo::Pid, &Process>,
            tree: &HashMap<sysinfo::Pid, Vec<sysinfo::Pid>>,
            users: &Users,
            cpu_count: f32,
        ) -> ProcessTreeNode {
            let process = processes[&pid];
            let mut children = Vec::new();
            let mut total_cpu = process.cpu_usage() / cpu_count;
            let mem = get_process_memory(pid);
            let mut total_mem = mem.rss;
            let mut total_virt = mem.virt;
            let mut total_shared = mem.shared;

            if let Some(child_pids) = tree.get(&pid) {
                for &child_pid in child_pids {
                    let child_node =
                        build_node(child_pid, depth + 1, processes, tree, users, cpu_count);
                    total_cpu += child_node.total_cpu;
                    total_mem += child_node.total_mem;
                    total_virt += child_node.total_virt;
                    total_shared += child_node.total_shared;
                    children.push(child_node);
                }
            }
            let user_name = process
                .user_id()
                .and_then(|uid| users.get_user_by_id(uid))
                .map(|u| u.name().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            ProcessTreeNode {
                pid,
                name: process.name().to_string_lossy().to_string(),
                user: user_name,
                cpu_usage: process.cpu_usage() / cpu_count,
                memory: mem.rss,
                virt_mem: mem.virt,
                shared_mem: mem.shared,
                depth,
                children,
                total_cpu,
                total_mem,
                total_virt,
                total_shared,
            }
        }
        let mut root_nodes = Vec::new();
        for pid in root_pids {
            root_nodes.push(build_node(
                pid,
                0,
                &filtered,
                &tree,
                &self.users_list,
                cpu_count,
            ));
        }
        root_nodes.sort_by(|a, b| {
            b.total_cpu
                .partial_cmp(&a.total_cpu)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        root_nodes
    }

    pub fn calculate_flattened_tree(&self) -> Vec<FlattenedTreeNode> {
        use crate::dashboard::utils::format_bytes;
        let mut flat = Vec::new();
        fn flatten_tree(node: &ProcessTreeNode, max_depth: u32, rows: &mut Vec<FlattenedTreeNode>) {
            let indent = "  ".repeat(node.depth as usize);
            let prefix = if node.depth > 0 { "└─ " } else { "" };
            let cpu_str = if node.depth < max_depth && !node.children.is_empty() {
                format!("({:.1}%)", node.total_cpu * 100.0)
            } else {
                format!("{:.1}%", node.cpu_usage * 100.0)
            };
            let mem_str = if node.depth < max_depth && !node.children.is_empty() {
                format!(
                    "{} ({})",
                    format_bytes(node.memory),
                    format_bytes(node.total_mem)
                )
            } else {
                format_bytes(node.memory)
            };
            let virt_str = if node.depth < max_depth && !node.children.is_empty() {
                format!(
                    "{} ({})",
                    format_bytes(node.virt_mem),
                    format_bytes(node.total_virt)
                )
            } else {
                format_bytes(node.virt_mem)
            };
            let shared_str = if node.depth < max_depth && !node.children.is_empty() {
                format!(
                    "{} ({})",
                    format_bytes(node.shared_mem),
                    format_bytes(node.total_shared)
                )
            } else {
                format_bytes(node.shared_mem)
            };

            rows.push(FlattenedTreeNode {
                pid: node.pid.to_string(),
                user: node.user.clone(),
                cpu: cpu_str,
                mem: mem_str,
                virt: virt_str,
                shared: shared_str,
                name: format!("{}{}{}", indent, prefix, node.name),
                depth: node.depth,
            });
            if node.depth + 1 < max_depth {
                for child in &node.children {
                    flatten_tree(child, max_depth, rows);
                }
            }
        }
        for root in &self.process_tree {
            flatten_tree(root, self.tree_expansion_depth, &mut flat);
        }
        flat
    }

    pub fn get_current_list_len(&self) -> usize {
        match self.tab_index {
            1 => self.disks.len(),
            2 => {
                if self.use_tree_view {
                    self.flattened_tree.len()
                } else {
                    self.sorted_processes.len()
                }
            }
            3 => self.sessions.len(),
            4 => self.interfaces.len(),
            5 => self.services.len(),
            6 => self.containers.len(),
            7 => self.sensors.len(),
            8 => 0,
            _ => 0,
        }
    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % self.tabs.len();
        self.selected_index = 0;
    }
    pub fn prev_tab(&mut self) {
        if self.tab_index > 0 {
            self.tab_index -= 1;
        } else {
            self.tab_index = self.tabs.len() - 1;
        }
        self.selected_index = 0;
    }
    pub fn on_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
    pub fn on_down(&mut self) {
        let len = self.get_current_list_len();
        if len > 0 && self.selected_index < len.saturating_sub(1) {
            self.selected_index += 1;
        }
    }
    pub fn on_page_up(&mut self) {
        let items_per_page = 20;
        if self.selected_index >= items_per_page {
            self.selected_index -= items_per_page;
        } else {
            self.selected_index = 0;
        }
    }
    pub fn on_page_down(&mut self) {
        let len = self.get_current_list_len();
        if len > 0 {
            let items_per_page = 20;
            let max_idx = len.saturating_sub(1);
            self.selected_index = (self.selected_index + items_per_page).min(max_idx);
        }
    }
}
