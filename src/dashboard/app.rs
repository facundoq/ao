use crate::config::Config;
pub use crate::config::ProcessSort;
use crate::dashboard::rss::{get_process_memory, get_rss};
use crate::os::detector::DetectedSystem;
use crate::os::{ContainerInfo, NetInterfaceInfo, SensorInfo, ServiceInfo, UserSessionInfo};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use sysinfo::{
    CpuRefreshKind, Disks, MemoryRefreshKind, Networks, Process, RefreshKind, System, Users,
};

pub struct App<'a> {
    pub system_info: System,
    pub disks: Disks,
    pub networks: Networks,
    pub users_list: Users,
    pub detected_system: &'a DetectedSystem,
    pub tab_index: usize,
    pub tabs: Vec<&'static str>,
    pub selected_index: usize,
    pub scroll_offset: usize,

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
    pub show_user_threads: bool,
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
    pub tick_rate: Duration,

    pub selected_process: Option<sysinfo::Pid>,
    pub process_details: Option<ProcessDetails>,
}

#[derive(Debug, Clone)]
pub struct ProcessResource {
    pub name: String,
    pub resource_type: String,
}

#[derive(Debug, Clone)]
pub struct ProcessDetails {
    pub pid: sysinfo::Pid,
    pub name: String,
    pub threads: Vec<(String, String)>, // (tid, name)
    pub resources: Vec<ProcessResource>,
    pub filtered_resources: Vec<ProcessResource>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub filter: String,
    pub is_searching: bool,
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
    pub thread_count: usize,
    pub fd_count: usize,
    pub depth: u32,
    pub children: Vec<ProcessTreeNode>,
    pub total_cpu: f32,
    pub total_mem: u64,
    pub total_virt: u64,
    pub total_shared: u64,
    pub total_threads: usize,
    pub total_fds: usize,
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
    pub executable: String,
    pub thread_count: usize,
    pub fd_count: usize,
}

#[derive(Debug, Clone)]
pub struct FlattenedTreeNode {
    pub pid: String,
    pub user: String,
    pub cpu: String,
    pub mem: String,
    pub virt: String,
    pub shared: String,
    pub threads: String,
    pub fds: String,
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
                "Process",
                "Storage",
                "User",
                "Network",
                "Service",
                "Virtualization",
                "Sensors",
                "Charts",
            ],
            selected_index: 0,
            scroll_offset: 0,
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
            process_sort: config.ui.process_sort,
            sort_descending: config.ui.process_sort_descending,
            use_tree_view: config.ui.process_use_tree_view,
            tree_expansion_depth: config.ui.process_tree_depth,
            show_only_current_user: config.ui.process_current_user_only,
            hide_kernel_processes: !config.ui.show_kernel_processes,
            show_user_threads: config.ui.show_user_threads,
            total_rss: 0,
            process_tree: Vec::new(),
            top_cpu_processes: Vec::new(),
            top_mem_processes: Vec::new(),
            sorted_processes: Vec::new(),
            flattened_tree: Vec::new(),
            last_process_refresh: Instant::now() - Duration::from_secs(11),
            refresh_interval: Duration::from_secs(10),
            max_temps: HashMap::new(),
            process_filter: config.ui.process_filter,
            is_filtering: false,
            tick_rate: Duration::from_millis(config.ui.refresh_rate_ms),
            selected_process: None,
            process_details: None,
        };
        app.refresh_users_list();
        app.on_tick();
        app
    }

    pub fn fetch_process_details(&mut self) {
        let pid = match self.tab_index {
            1 => {
                if self.use_tree_view && self.tree_expansion_depth > 0 {
                    self.flattened_tree
                        .get(self.selected_index)
                        .and_then(|node| node.pid.parse::<usize>().ok())
                        .map(sysinfo::Pid::from)
                } else {
                    self.sorted_processes
                        .get(self.selected_index)
                        .map(|p| p.pid)
                }
            }
            _ => None,
        };

        if let Some(pid) = pid {
            let mut threads = Vec::new();
            let mut resources = Vec::new();

            // Fetch threads
            if let Ok(entries) = std::fs::read_dir(format!("/proc/{}/task", pid)) {
                for entry in entries.flatten() {
                    let tid = entry.file_name().to_string_lossy().to_string();
                    let comm_path = entry.path().join("comm");
                    let name = std::fs::read_to_string(comm_path)
                        .unwrap_or_else(|_| "unknown".to_string())
                        .trim()
                        .to_string();
                    threads.push((tid, name));
                }
            }

            // Fetch FDs
            if let Ok(entries) = std::fs::read_dir(format!("/proc/{}/fd", pid)) {
                for entry in entries.flatten() {
                    if let Ok(target) = std::fs::read_link(entry.path()) {
                        let target_str = target.to_string_lossy().to_string();
                        let resource_type = if target_str.starts_with("socket:[") {
                            "Socket".to_string()
                        } else if target_str.starts_with("pipe:[") {
                            "Pipe".to_string()
                        } else if target_str.starts_with("anon_inode:") {
                            "Anon".to_string()
                        } else {
                            "File".to_string()
                        };
                        resources.push(ProcessResource {
                            name: target_str,
                            resource_type,
                        });
                    }
                }
            }

            let name = self
                .system_info
                .process(pid)
                .map(|p| p.name().to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let mut details = ProcessDetails {
                pid,
                name,
                threads,
                resources,
                filtered_resources: Vec::new(),
                selected_index: 0,
                scroll_offset: 0,
                filter: String::new(),
                is_searching: false,
            };
            details.filtered_resources = details.resources.clone();
            self.process_details = Some(details);
        }
    }

    pub fn update_details_filter(&mut self) {
        if let Some(ref mut details) = self.process_details {
            let filter = details.filter.to_lowercase();
            details.filtered_resources = details
                .resources
                .iter()
                .filter(|r| {
                    r.name.to_lowercase().contains(&filter)
                        || r.resource_type.to_lowercase().contains(&filter)
                })
                .cloned()
                .collect();
            details.selected_index = 0;
            details.scroll_offset = 0;
        }
    }

    pub fn toggle_details_filter_mode(&mut self) {
        if let Some(ref mut details) = self.process_details {
            details.is_searching = !details.is_searching;
        }
    }

    pub fn details_on_up(&mut self) {
        if let Some(ref mut details) = self.process_details
            && details.selected_index > 0
        {
            details.selected_index -= 1;
            if details.selected_index < details.scroll_offset {
                details.scroll_offset = details.selected_index;
            }
        }
    }

    pub fn details_on_down(&mut self) {
        if let Some(ref mut details) = self.process_details {
            let len = details.filtered_resources.len();
            if len > 0 && details.selected_index < len.saturating_sub(1) {
                details.selected_index += 1;
                if details.selected_index >= details.scroll_offset + 20 {
                    details.scroll_offset += 1;
                }
            }
        }
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
        self.system_info.refresh_cpu_all();
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
        let mem_available = self.system_info.available_memory() as f64;
        let mem_used_actual = mem_total - mem_available;
        let mem_percent = if mem_total > 0.0 {
            (mem_used_actual / mem_total) * 100.0
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

        let all_summaries = self.get_sorted_processes(false); // Unfiltered for Overview

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

        self.sorted_processes = self.get_sorted_processes(true); // Filtered for Tab
        self.flattened_tree = self.calculate_flattened_tree();
    }

    pub fn get_top_cpu_processes(&self, count: usize) -> Vec<ProcessSummary> {
        let mut all = Vec::new();
        fn flatten(node: &ProcessTreeNode, out: &mut Vec<ProcessSummary>) {
            out.push(ProcessSummary {
                pid: node.pid,
                name: node.name.clone(),
                user: node.user.clone(),
                memory: node.memory, // Changed from total_mem to be consistent with individual sort
                virt_mem: node.virt_mem,
                shared_mem: node.shared_mem,
                cpu: node.cpu_usage, // Changed from total_cpu to be consistent with individual sort
                command: "".to_string(),
                executable: "".to_string(),
                thread_count: 0,
                fd_count: 0,
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
                memory: node.memory,
                virt_mem: node.virt_mem,
                shared_mem: node.shared_mem,
                cpu: node.cpu_usage,
                command: "".to_string(),
                executable: "".to_string(),
                thread_count: 0,
                fd_count: 0,
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

    pub fn get_sorted_processes(&self, apply_filter: bool) -> Vec<ProcessSummary> {
        let current_uid = unsafe { libc::getuid() };
        let mut processes: Vec<&Process> = self
            .system_info
            .processes()
            .values()
            .filter(|p| {
                if !self.show_user_threads && p.thread_kind().is_some() {
                    return false;
                }
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
                if apply_filter && !self.process_filter.is_empty() {
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
                        .cmd()
                        .iter()
                        .map(|s| s.to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                    executable: p
                        .exe()
                        .map(|e| e.to_string_lossy().to_string())
                        .unwrap_or_else(|| "".to_string()),
                    thread_count: 0,
                    fd_count: 0,
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
            if !self.show_user_threads && process.thread_kind().is_some() {
                continue;
            }
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

        struct SortConfig {
            by: ProcessSort,
            descending: bool,
        }

        fn build_node(
            pid: sysinfo::Pid,
            depth: u32,
            processes: &HashMap<sysinfo::Pid, &Process>,
            tree: &HashMap<sysinfo::Pid, Vec<sysinfo::Pid>>,
            users: &Users,
            cpu_count: f32,
            sort: &SortConfig,
        ) -> ProcessTreeNode {
            let process = processes[&pid];
            let mut children = Vec::new();
            let mem = get_process_memory(pid);
            let mut total_cpu = process.cpu_usage() / cpu_count;
            let mut total_mem = mem.rss;
            let mut total_virt = mem.virt;
            let mut total_shared = mem.shared;

            if let Some(child_pids) = tree.get(&pid) {
                for &child_pid in child_pids {
                    let child_node = build_node(
                        child_pid,
                        depth + 1,
                        processes,
                        tree,
                        users,
                        cpu_count,
                        sort,
                    );
                    total_cpu += child_node.total_cpu;
                    total_mem += child_node.total_mem;
                    total_virt += child_node.total_virt;
                    total_shared += child_node.total_shared;
                    children.push(child_node);
                }
            }

            // Sort children
            children.sort_by(|a, b| {
                let res = match sort.by {
                    ProcessSort::Pid => a.pid.cmp(&b.pid),
                    ProcessSort::Cpu => a
                        .total_cpu
                        .partial_cmp(&b.total_cpu)
                        .unwrap_or(std::cmp::Ordering::Equal),
                    ProcessSort::Mem => a.total_mem.cmp(&b.total_mem),
                    ProcessSort::Name => a.name.cmp(&b.name),
                    ProcessSort::User => a.user.cmp(&b.user),
                };
                if sort.descending { res.reverse() } else { res }
            });

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
                thread_count: 0,
                fd_count: 0,
                depth,
                children,
                total_cpu,
                total_mem,
                total_virt,
                total_shared,
                total_threads: 0,
                total_fds: 0,
            }
        }

        let sort_config = SortConfig {
            by: self.process_sort,
            descending: self.sort_descending,
        };

        let mut root_nodes = Vec::new();
        for pid in root_pids {
            root_nodes.push(build_node(
                pid,
                0,
                &filtered,
                &tree,
                &self.users_list,
                cpu_count,
                &sort_config,
            ));
        }
        root_nodes.sort_by(|a, b| {
            let res = match self.process_sort {
                ProcessSort::Pid => a.pid.cmp(&b.pid),
                ProcessSort::Cpu => a
                    .total_cpu
                    .partial_cmp(&b.total_cpu)
                    .unwrap_or(std::cmp::Ordering::Equal),
                ProcessSort::Mem => a.total_mem.cmp(&b.total_mem),
                ProcessSort::Name => a.name.cmp(&b.name),
                ProcessSort::User => a.user.cmp(&b.user),
            };
            if self.sort_descending {
                res.reverse()
            } else {
                res
            }
        });
        root_nodes
    }

    pub fn calculate_flattened_tree(&self) -> Vec<FlattenedTreeNode> {
        use crate::dashboard::utils::format_bytes;
        let mut flat = Vec::new();
        fn flatten_tree(node: &ProcessTreeNode, max_depth: u32, rows: &mut Vec<FlattenedTreeNode>) {
            let indent = "  ".repeat(node.depth as usize);
            let prefix = if node.depth > 0 { "└─ " } else { "" };

            let is_expanded = node.depth + 1 < max_depth;
            let has_children = !node.children.is_empty();
            let show_total = !is_expanded && has_children;

            let cpu_str = if show_total {
                format!("{:.1}%", node.total_cpu)
            } else {
                format!("{:.1}%", node.cpu_usage)
            };
            let mem_str = if show_total {
                format_bytes(node.total_mem)
            } else {
                format_bytes(node.memory)
            };
            let virt_str = if show_total {
                format_bytes(node.total_virt)
            } else {
                format_bytes(node.virt_mem)
            };
            let shared_str = if show_total {
                format_bytes(node.total_shared)
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
                threads: "".to_string(),
                fds: "".to_string(),
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
            1 => {
                if self.use_tree_view && self.tree_expansion_depth > 0 {
                    self.flattened_tree.len()
                } else {
                    self.sorted_processes.len()
                }
            }
            2 => self.disks.len(),
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
        self.scroll_offset = 0;
    }
    pub fn prev_tab(&mut self) {
        if self.tab_index > 0 {
            self.tab_index -= 1;
        } else {
            self.tab_index = self.tabs.len() - 1;
        }
        self.selected_index = 0;
        self.scroll_offset = 0;
    }
    pub fn on_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
        }
    }
    pub fn on_down(&mut self) {
        let len = self.get_current_list_len();
        if len > 0 && self.selected_index < len.saturating_sub(1) {
            self.selected_index += 1;
            // Simple heuristic: if we are at the bottom of what we think is a page, scroll
            if self.selected_index >= self.scroll_offset + 20 {
                self.scroll_offset += 1;
            }
        }
    }
    pub fn on_page_up(&mut self) {
        let items_per_page = 20;
        if self.selected_index >= items_per_page {
            self.selected_index -= items_per_page;
        } else {
            self.selected_index = 0;
        }
        self.scroll_offset = self.selected_index;
    }
    pub fn on_page_down(&mut self) {
        let len = self.get_current_list_len();
        if len > 0 {
            let items_per_page = 20;
            let max_idx = len.saturating_sub(1);
            self.selected_index = (self.selected_index + items_per_page).min(max_idx);
            self.scroll_offset = self.selected_index;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::os::detector::detect_system;

    fn setup_test_app() -> App<'static> {
        // We need a DetectedSystem, but we don't want to actually detect it if possible.
        // For now, let's just use the real one if we are on Linux, or bail.
        let system = Box::leak(Box::new(detect_system().unwrap()));
        App::new(system)
    }

    #[test]
    fn test_tab_navigation() {
        let mut app = setup_test_app();
        assert_eq!(app.tab_index, 0);
        app.next_tab();
        assert_eq!(app.tab_index, 1);
        app.selected_index = 10; // Mock some selection
        app.scroll_offset = 5;
        app.next_tab();
        assert_eq!(app.tab_index, 2);
        assert_eq!(app.selected_index, 0); // Should reset
        assert_eq!(app.scroll_offset, 0); // Should reset
        app.prev_tab();
        assert_eq!(app.tab_index, 1);
    }

    #[test]
    fn test_selection_logic() {
        let mut app = setup_test_app();
        app.tab_index = 1; // Processes
        // Mock some processes
        app.sorted_processes = vec![
            ProcessSummary {
                pid: sysinfo::Pid::from(1),
                name: "test".into(),
                user: "user".into(),
                memory: 0,
                virt_mem: 0,
                shared_mem: 0,
                cpu: 0.0,
                command: "".into(),
                executable: "".into(),
                thread_count: 0,
                fd_count: 0,
            };
            50
        ];

        assert_eq!(app.selected_index, 0);
        app.on_down();
        assert_eq!(app.selected_index, 1);
        app.on_up();
        assert_eq!(app.selected_index, 0);

        // Test scrolling heuristic
        for _ in 0..25 {
            app.on_down();
        }
        assert_eq!(app.selected_index, 25);
        assert!(app.scroll_offset > 0);

        app.on_page_up();
        assert_eq!(app.selected_index, 5);
        assert_eq!(app.scroll_offset, 5);
    }
}
