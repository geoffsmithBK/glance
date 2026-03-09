/// Truncate a string to a maximum length, adding ellipsis if needed
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s[..max_len].to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Get the unit suffix for large numbers (KB, MB, GB)
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    match bytes {
        b if b >= GB => format!("{:.2} GB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.2} MB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.2} KB", b as f64 / KB as f64),
        _ => format!("{} B", bytes),
    }
}

/// Get ASCII bar representation for a percentage (0-100)
pub fn percentage_bar(pct: f32, max_width: usize) -> String {
    let filled = ((pct / 100.0) * max_width as f32).round() as usize;
    let filled = filled.min(max_width);
    let empty = max_width - filled;

    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("short", 10), "short");
        assert_eq!(truncate_str("very long string", 10), "very lo...");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_percentage_bar() {
        assert_eq!(percentage_bar(100.0, 10), "██████████");
        assert_eq!(percentage_bar(50.0, 10), "█████░░░░░");
        assert_eq!(percentage_bar(0.0, 10), "░░░░░░░░░░");
    }
}
