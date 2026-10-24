#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryStats {
    pub rss: u64,
    pub virt: u64,
    pub shared: u64,
}

fn parse_statm(content: &str, page_size: u64) -> Option<MemoryStats> {
    let fields: Vec<&str> = content.split_whitespace().collect();
    if fields.len() >= 3 {
        let virt = fields[0].parse::<u64>().unwrap_or(0) * page_size;
        let rss = fields[1].parse::<u64>().unwrap_or(0) * page_size;
        let shared = fields[2].parse::<u64>().unwrap_or(0) * page_size;
        return Some(MemoryStats { rss, virt, shared });
    }
    None
}

fn get_process_memory_from_path(path: &std::path::Path, page_size: u64) -> MemoryStats {
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Some(stats) = parse_statm(&content, page_size) {
            return stats;
        }
    }
    MemoryStats {
        rss: 0,
        virt: 0,
        shared: 0,
    }
}

pub fn get_process_memory(pid: sysinfo::Pid) -> MemoryStats {
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;
    let path = format!("/proc/{}/statm", pid);
    get_process_memory_from_path(std::path::Path::new(&path), page_size)
}

pub fn get_rss(pid: sysinfo::Pid) -> u64 {
    get_process_memory(pid).rss
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_statm_valid() {
        let content = "100 50 25 1 0 0 0";
        let page_size = 4096;
        let expected = MemoryStats {
            virt: 100 * 4096,
            rss: 50 * 4096,
            shared: 25 * 4096,
        };
        assert_eq!(parse_statm(content, page_size), Some(expected));
    }

    #[test]
    fn test_parse_statm_invalid_numbers() {
        let content = "abc def ghi 1 0 0 0";
        let page_size = 4096;
        let expected = MemoryStats {
            virt: 0,
            rss: 0,
            shared: 0,
        };
        assert_eq!(parse_statm(content, page_size), Some(expected));
    }

    #[test]
    fn test_parse_statm_too_few_fields() {
        let content = "100 50";
        let page_size = 4096;
        assert_eq!(parse_statm(content, page_size), None);
    }

    #[test]
    fn test_get_process_memory_from_path_valid() {
        let dir = std::env::temp_dir();
        let file_path = dir.join("statm_test_valid");

        // Write mock data to a temporary file
        let mut file = std::fs::File::create(&file_path).unwrap();
        writeln!(file, "100 50 25 1 0 0 0").unwrap();

        let stats = get_process_memory_from_path(&file_path, 4096);
        let expected = MemoryStats {
            virt: 100 * 4096,
            rss: 50 * 4096,
            shared: 25 * 4096,
        };

        assert_eq!(stats, expected);

        // Clean up
        std::fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_get_process_memory_from_path_nonexistent() {
        let path = std::path::Path::new("/path/that/does/not/exist");
        let stats = get_process_memory_from_path(path, 4096);

        let expected = MemoryStats {
            virt: 0,
            rss: 0,
            shared: 0,
        };

        assert_eq!(stats, expected);
    }
}
