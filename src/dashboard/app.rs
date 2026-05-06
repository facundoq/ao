use crate::os::detector::DetectedSystem;
use crate::os::{ContainerInfo, NetInterfaceInfo, ServiceInfo, UserSessionInfo};
use std::collections::HashMap;
use std::time::Instant;
use sysinfo::{
    CpuRefreshKind, Disks, MemoryRefreshKind, Networks, Process, RefreshKind, System, Users,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSort {
    Pid,
    Cpu,
    Mem,
    Name,
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

    // Tab-specific data
    pub sessions: Vec<UserSessionInfo>,
    pub users: Vec<(String, bool)>,
    pub interfaces: Vec<NetInterfaceInfo>,
    pub services: Vec<ServiceInfo>,
    pub containers: Vec<ContainerInfo>,

    // Network speed tracking
    pub last_tick_time: Instant,
    pub prev_network_data: HashMap<String, (u64, u64)>, // interface -> (rx, tx)
    pub network_speeds: HashMap<String, (u64, u64)>,    // interface -> (rx_speed, tx_speed)

    // Process sorting and view
    pub process_sort: ProcessSort,
    pub sort_descending: bool,
    pub use_tree_view: bool,
    pub tree_expansion_depth: u32,
}

#[derive(Debug, Clone)]
pub struct ProcessTreeNode {
    pub pid: sysinfo::Pid,
    pub name: String,
    pub user: String,
    pub cpu_usage: f32,
    pub memory: u64,
    pub depth: u32,
    pub children: Vec<ProcessTreeNode>,
    pub total_cpu: f32,
    pub total_mem: u64,
}

impl<'a> App<'a> {
    pub fn new(detected_system: &'a DetectedSystem) -> Self {
        let mut system_info = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(sysinfo::ProcessRefreshKind::everything()),
        );
        system_info.refresh_all();

        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        let users_list = Users::new_with_refreshed_list();

        let mut sessions = detected_system
            .user
            .get_sessions(None, true, Some(50))
            .unwrap_or_default();
        sessions.reverse();
        let interfaces = detected_system.net.get_interfaces().unwrap_or_default();
        let services = detected_system
            .svc
            .get_all_services_info()
            .unwrap_or_default();
        let containers = detected_system.virt.get_containers().unwrap_or_default();

        let mut prev_network_data = HashMap::new();
        for (name, network) in &networks {
            prev_network_data.insert(name.clone(), (network.received(), network.transmitted()));
        }

        let mut app = Self {
            system_info,
            disks,
            networks,
            users_list,
            detected_system,
            tab_index: 0,
            tabs: vec![
                "Overview",
                "Process",
                "User",
                "Network",
                "Service",
                "Virtualization",
            ],
            selected_index: 0,
            sessions,
            users: Vec::new(),
            interfaces,
            services,
            containers,
            last_tick_time: Instant::now(),
            prev_network_data,
            network_speeds: HashMap::new(),
            process_sort: ProcessSort::Cpu,
            sort_descending: true,
            use_tree_view: true,
            tree_expansion_depth: 2,
        };
        app.refresh_users();
        app
    }

    pub fn refresh_users(&mut self) {
        let mut users = Vec::new();
        for user in &self.users_list {
            let uid_str = user.id().to_string();
            let uid = uid_str.parse::<u32>().unwrap_or(0);
            users.push((user.name().to_string(), uid < 1000));
        }

        // Sort: normal users first, then system users. Within groups, sort by name.
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

        self.system_info.refresh_all();
        self.disks.refresh(true);

        // Update network speeds
        self.networks.refresh(true);

        for (name, network) in &self.networks {
            let rx = network.received();
            let tx = network.transmitted();

            if let Some((old_rx, old_tx)) = self.prev_network_data.get(name) {
                let rx_delta = rx.saturating_sub(*old_rx);
                let tx_delta = tx.saturating_sub(*old_tx);
                let rx_speed = (rx_delta as f64 / elapsed) as u64;
                let tx_speed = (tx_delta as f64 / elapsed) as u64;
                self.network_speeds
                    .insert(name.clone(), (rx_speed, tx_speed));
            }
            self.prev_network_data.insert(name.clone(), (rx, tx));
        }

        // Refresh based on tab
        match self.tab_index {
            2 => {
                if let Ok(mut s) = self.detected_system.user.get_sessions(None, true, Some(50)) {
                    s.reverse();
                    self.sessions = s;
                }
            }
            3 => {
                if let Ok(i) = self.detected_system.net.get_interfaces() {
                    self.interfaces = i;
                }
            }
            4 => {
                if let Ok(s) = self.detected_system.svc.get_all_services_info() {
                    self.services = s;
                }
            }
            5 => {
                if let Ok(c) = self.detected_system.virt.get_containers() {
                    self.containers = c;
                }
            }
            _ => {}
        }
    }

    pub fn get_sorted_processes(&self) -> Vec<&Process> {
        let mut processes: Vec<&Process> = self.system_info.processes().values().collect();
        processes.sort_by(|a, b| {
            let res = match self.process_sort {
                ProcessSort::Pid => a.pid().cmp(&b.pid()),
                ProcessSort::Cpu => a
                    .cpu_usage()
                    .partial_cmp(&b.cpu_usage())
                    .unwrap_or(std::cmp::Ordering::Equal),
                ProcessSort::Mem => a.memory().cmp(&b.memory()),
                ProcessSort::Name => a.name().cmp(b.name()),
            };
            if self.sort_descending {
                res.reverse()
            } else {
                res
            }
        });
        processes
    }

    pub fn get_process_tree(&self) -> Vec<ProcessTreeNode> {
        let processes = self.system_info.processes();

        // Build the tree and calculate totals
        let mut root_pids = Vec::new();

        for (pid, process) in processes {
            if let Some(parent_pid) = process.parent() {
                if !processes.contains_key(&parent_pid) {
                    root_pids.push(*pid);
                }
            } else {
                root_pids.push(*pid);
            }
        }

        // Actually, a simpler way to get a flat list that looks like a tree:
        let roots: Vec<sysinfo::Pid> = root_pids;

        // Refined tree building logic:
        let mut tree: HashMap<sysinfo::Pid, Vec<sysinfo::Pid>> = HashMap::new();
        for (pid, process) in processes {
            if let Some(parent_pid) = process.parent() {
                tree.entry(parent_pid).or_default().push(*pid);
            }
        }

        fn build_node(
            pid: sysinfo::Pid,
            depth: u32,
            processes: &HashMap<sysinfo::Pid, Process>,
            tree: &HashMap<sysinfo::Pid, Vec<sysinfo::Pid>>,
            users: &Users,
        ) -> ProcessTreeNode {
            let process = &processes[&pid];
            let mut children = Vec::new();
            let mut total_cpu = process.cpu_usage();
            let mut total_mem = process.memory();

            if let Some(child_pids) = tree.get(&pid) {
                for &child_pid in child_pids {
                    let child_node = build_node(child_pid, depth + 1, processes, tree, users);
                    total_cpu += child_node.total_cpu;
                    total_mem += child_node.total_mem;
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
                cpu_usage: process.cpu_usage(),
                memory: process.memory(),
                depth,
                children,
                total_cpu,
                total_mem,
            }
        }

        let mut root_nodes = Vec::new();
        for pid in roots {
            if processes.contains_key(&pid) {
                root_nodes.push(build_node(pid, 0, processes, &tree, &self.users_list));
            }
        }

        // Sort roots by total_cpu descending by default
        root_nodes.sort_by(|a, b| {
            b.total_cpu
                .partial_cmp(&a.total_cpu)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        root_nodes
    }

    pub fn get_top_cpu_processes(&self, count: usize) -> Vec<&Process> {
        let mut processes: Vec<&Process> = self.system_info.processes().values().collect();
        processes.sort_by(|a, b| {
            b.cpu_usage()
                .partial_cmp(&a.cpu_usage())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        processes.into_iter().take(count).collect()
    }

    pub fn get_top_mem_processes(&self, count: usize) -> Vec<&Process> {
        let mut processes: Vec<&Process> = self.system_info.processes().values().collect();
        processes.sort_by_key(|b| std::cmp::Reverse(b.memory()));
        processes.into_iter().take(count).collect()
    }

    pub fn get_flattened_tree_len(&self) -> usize {
        let roots = self.get_process_tree();
        let mut count = 0;

        fn count_nodes(node: &ProcessTreeNode, max_depth: u32, count: &mut usize) {
            *count += 1;
            if node.depth + 1 < max_depth {
                for child in &node.children {
                    count_nodes(child, max_depth, count);
                }
            }
        }

        for root in roots {
            count_nodes(&root, self.tree_expansion_depth, &mut count);
        }
        count
    }

    pub fn get_current_list_len(&self) -> usize {
        match self.tab_index {
            1 => {
                if self.use_tree_view {
                    self.get_flattened_tree_len()
                } else {
                    self.system_info.processes().len()
                }
            }
            2 => self.sessions.len().max(self.users.len()),
            3 => self.interfaces.len(),
            4 => self.services.len(),
            5 => self.containers.len(),
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
        if self.selected_index >= 20 {
            self.selected_index -= 20;
        } else {
            self.selected_index = 0;
        }
    }

    pub fn on_page_down(&mut self) {
        let len = self.get_current_list_len();
        if len > 0 {
            let max_idx = len.saturating_sub(1);
            self.selected_index = (self.selected_index + 20).min(max_idx);
        }
    }
}
