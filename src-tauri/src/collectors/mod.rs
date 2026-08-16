#[cfg(windows)]
pub mod connections;
#[cfg(windows)]
mod network;
#[cfg(windows)]
pub mod processes;
#[cfg(windows)]
pub mod services;
#[cfg(windows)]
mod system;

use crate::models::SystemOverview;

#[cfg(windows)]
pub fn collect_overview() -> Result<SystemOverview, String> {
    system::collect(network::collect())
}

#[cfg(not(windows))]
pub fn collect_overview() -> Result<SystemOverview, String> {
    Err("EDY Sentinel system collection is available on Windows only".into())
}
