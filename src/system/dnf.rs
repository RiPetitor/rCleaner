pub async fn list_installed() -> crate::error::Result<Vec<String>> {
    todo!("Implement DNF package listing")
}

pub async fn remove_packages(packages: &[String], dry_run: bool) -> crate::error::Result<()> {
    todo!("Implement DNF package removal")
}
