cat << 'INNER_EOF' > src/dashboard/utils.rs
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
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "    0  B");
        assert_eq!(format_bytes(512), "  512  B");
        assert_eq!(format_bytes(1023), " 1023  B");
        assert_eq!(format_bytes(1024), "  1.0 KB");
        assert_eq!(format_bytes(1536), "  1.5 KB");
        assert_eq!(format_bytes(1024 * 1024 - 1), "1024.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "  1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "  1.0 GB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 1024), "  1.0 TB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 1024 * 1024), "  1.0 PB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 1024 * 1024 * 2), "  2.0 PB");
    }

    #[test]
    fn test_get_user_color_determinism() {
        assert_eq!(get_user_color("root"), get_user_color("root"));
        assert_eq!(get_user_color("admin"), get_user_color("admin"));
        assert_eq!(get_user_color(""), get_user_color(""));
    }

    #[test]
    fn test_get_user_color_mapping() {
        // empty string: sum = 0, 0 % 6 = 0 -> Red
        assert_eq!(get_user_color(""), Color::Red);

        // '1': sum = 49, 49 % 6 = 1 -> Green
        assert_eq!(get_user_color("1"), Color::Green);

        // "root": sum = 114+111+111+116 = 452, 452 % 6 = 2 -> Yellow
        assert_eq!(get_user_color("root"), Color::Yellow);

        // "user": sum = 117+115+101+114 = 447, 447 % 6 = 3 -> Blue
        assert_eq!(get_user_color("user"), Color::Blue);

        // 'x': sum = 120, 120 % 6 = 0 -> Red (we need a Magenta test)
        // Magenta is index 4. So sum % 6 = 4.
        // 'v' is 118, 118 % 6 = 4 -> Magenta
        assert_eq!(get_user_color("v"), Color::Magenta);

        // "admin": sum = 97+100+109+105+110 = 521, 521 % 6 = 5 -> Cyan
        assert_eq!(get_user_color("admin"), Color::Cyan);

        // "guest": sum = 103+117+101+115+116 = 552, 552 % 6 = 0 -> Red
        assert_eq!(get_user_color("guest"), Color::Red);
    }
}
INNER_EOF
git add src/dashboard/utils.rs
git -c core.editor=true rebase --continue
