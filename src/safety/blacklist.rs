use crate::error::Result;
use std::path::Path;

#[allow(dead_code)]
pub fn load_blacklist(path: &Path) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    Ok(parse_list(&content))
}

#[allow(dead_code)]
pub fn save_blacklist(path: &Path, blacklist: &[String]) -> Result<()> {
    let content = blacklist
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(path, format!("{content}\n"))?;
    Ok(())
}

#[allow(dead_code)]
fn parse_list(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(String::from)
        .collect()
}
