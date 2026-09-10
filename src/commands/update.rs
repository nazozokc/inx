use anyhow::Result;

use crate::utils::registry::update_registry;

/// Run `inx update`.
pub fn run() -> Result<()> {
    update_registry()?;
    println!("Registry updated successfully");
    Ok(())
}