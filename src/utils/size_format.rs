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

pub fn parse_size_string(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (num_str, unit_str) = split_number_and_unit(trimmed);
    let number: f64 = num_str.parse().ok()?;

    let mut unit = unit_str.trim().to_lowercase();
    unit = unit.replace("bytes", "b").replace("byte", "b");
    unit = unit
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric())
        .to_string();

    let factor = match unit.as_str() {
        "" | "b" => 1.0,
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        "p" | "pb" | "pib" => 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };

    Some((number * factor).round() as u64)
}

fn split_number_and_unit(value: &str) -> (String, String) {
    let mut parts = value.split_whitespace();
    let first = parts.next().unwrap_or("");
    let rest = parts.collect::<Vec<_>>().join("");

    let (number, unit) = if rest.is_empty() {
        split_inline_number_unit(first)
    } else {
        (first.to_string(), rest)
    };

    (number.replace(',', "."), unit)
}

fn split_inline_number_unit(value: &str) -> (String, String) {
    let mut number = String::new();
    let mut unit = String::new();
    for ch in value.chars() {
        if ch.is_ascii_digit() || ch == '.' || ch == ',' {
            if unit.is_empty() {
                number.push(ch);
            } else {
                unit.push(ch);
            }
        } else {
            unit.push(ch);
        }
    }
    (number, unit)
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

    #[test]
    fn test_parse_size_string() {
        assert_eq!(parse_size_string("123"), Some(123));
        assert_eq!(parse_size_string("1 KB"), Some(1024));
        assert_eq!(parse_size_string("1.5MB"), Some(1572864));
        assert_eq!(parse_size_string("2GiB"), Some(2147483648));
    }
}
