use ratatui::style::Color;

pub fn format_uptime(seconds: u64) -> String {
    let d = seconds / 86400;
    let h = (seconds % 86400) / 3600;
    let m = (seconds % 3600) / 60;
    if d > 0 {
        format!("{}d {}h {}m", d, h, m)
    } else if h > 0 {
        format!("{}h {}m", h, m)
    } else {
        format!("{}m", m)
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    const PB: u64 = TB * 1024;

    if bytes >= PB {
        format!("{:>5.1} PB", bytes as f64 / PB as f64)
    } else if bytes >= TB {
        format!("{:>5.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:>5.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:>5.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:>5.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{:>5}  B", bytes)
    }
}

pub fn make_bar(percent: u16, width: u16) -> String {
    let p = (percent as f32 / 100.0).clamp(0.0, 1.0);
    let filled = (p * width as f32).round() as u16;
    let mut bar = String::from("[");
    for i in 0..width {
        if i < filled {
            bar.push('█');
        } else {
            bar.push(' ');
        }
    }
    bar.push(']');
    bar
}

pub fn get_user_color(user: &str) -> Color {
    let hash = user.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
    let colors = [
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
    ];
    colors[(hash as usize) % colors.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(0), "0m");
        assert_eq!(format_uptime(59), "0m");
        assert_eq!(format_uptime(60), "1m");
        assert_eq!(format_uptime(3599), "59m");
        assert_eq!(format_uptime(3600), "1h 0m");
        assert_eq!(format_uptime(3660), "1h 1m");
        assert_eq!(format_uptime(86399), "23h 59m");
        assert_eq!(format_uptime(86400), "1d 0h 0m");
        assert_eq!(format_uptime(90060), "1d 1h 1m");
    }
}
