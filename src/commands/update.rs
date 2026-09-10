use anyhow::Result;

use crate::utils::registry::update_registry;

/// Run `ox update`.
pub fn run() -> Result<()> {
    update_registry()?;
    println!("Registry updated successfully");
    Ok(())
}