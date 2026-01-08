pub fn check_dependencies(package: &str) -> crate::error::Result<Vec<String>> {
    todo!("Implement dependency checking")
}

pub fn is_safe_to_remove(package: &str, installed_packages: &[String]) -> bool {
    todo!("Implement safety check")
}
