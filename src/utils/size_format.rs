pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_index = 0usize;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

pub fn format_percentage(part: u64, total: u64) -> String {
    if total == 0 {
        "0%".to_string()
    } else {
        format!("{}%", (part * 100) / total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0.00 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_percentage() {
        assert_eq!(format_percentage(50, 100), "50%");
        assert_eq!(format_percentage(25, 100), "25%");
        assert_eq!(format_percentage(0, 100), "0%");
        assert_eq!(format_percentage(100, 0), "0%");
    }
}
