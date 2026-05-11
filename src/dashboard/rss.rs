#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryStats {
    pub rss: u64,
    pub virt: u64,
    pub shared: u64,
}

pub fn get_process_memory(pid: sysinfo::Pid) -> MemoryStats {
    let path = format!("/proc/{}/statm", pid);
    if let Ok(content) = std::fs::read_to_string(path) {
        let fields: Vec<&str> = content.split_whitespace().collect();
        if fields.len() >= 3 {
            let virt = fields[0].parse::<u64>().unwrap_or(0) * 4096;
            let rss = fields[1].parse::<u64>().unwrap_or(0) * 4096;
            let shared = fields[2].parse::<u64>().unwrap_or(0) * 4096;
            return MemoryStats { rss, virt, shared };
        }
    }
    MemoryStats {
        rss: 0,
        virt: 0,
        shared: 0,
    }
}

pub fn get_rss(pid: sysinfo::Pid) -> u64 {
    get_process_memory(pid).rss
}
