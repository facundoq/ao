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
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn make_bar(percent: u16, width: u16) -> String {
    let filled = (percent as f32 / 100.0 * width as f32).round() as u16;
    let mut bar = String::from("[");
    for i in 0..width {
        if i < filled {
            bar.push('■');
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
