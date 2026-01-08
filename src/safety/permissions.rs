pub fn check_permissions() -> crate::error::Result<bool> {
    Ok(crate::utils::command::is_root())
}
