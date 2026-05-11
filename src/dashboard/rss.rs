pub fn get_rss(pid: sysinfo::Pid) -> u64 {
    let path = format!("/proc/{}/statm", pid);
    if let Ok(content) = std::fs::read_to_string(path) {
        let fields: Vec<&str> = content.split_whitespace().collect();
        if fields.len() >= 2
            && let Ok(rss_pages) = fields[1].parse::<u64>()
        {
            return rss_pages * 4096;
        }
    }
    0
}
