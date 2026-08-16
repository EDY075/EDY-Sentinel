//! Safe native smoke fixture for Sprint 2B.
//!
//! The binary performs no I/O, network access, persistence, or system changes.
//! It only remains alive long enough for the normal Windows collector cadence to
//! observe an unsigned executable running from the user's temporary directory.

fn main() {
    std::thread::sleep(std::time::Duration::from_secs(45));
}
