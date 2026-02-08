//! Platform-specific utilities and checks.

use std::io;
use std::path::Path;

/// Result of platform limits check.
#[derive(Debug)]
#[allow(dead_code)]
pub struct PlatformLimits {
    /// The current limit value.
    pub current_limit: Option<u64>,
    /// Whether the limit may be insufficient for the given count.
    pub may_be_insufficient: bool,
    /// Warning message if any.
    pub warning: Option<String>,
}

/// Check inotify user watches limit on Linux.
/// Returns information about whether the limit may be insufficient for watching
/// the given number of directories.
///
/// On non-Linux systems, this always returns Ok with no warning.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn check_inotify_limit(estimated_directories: usize) -> PlatformLimits {
    #[cfg(target_os = "linux")]
    {
        let limit = read_inotify_max_user_watches();

        // Estimate: each directory needs at least one watch.
        // Add 20% buffer for safety.
        let needed = (estimated_directories as f64 * 1.2) as u64;

        if let Some(max_watches) = limit {
            if needed > max_watches {
                return PlatformLimits {
                    current_limit: Some(max_watches),
                    may_be_insufficient: true,
                    warning: Some(format!(
                        "Warning: Linux inotify limit ({max_watches} watches) may be insufficient.\n\
                         You are trying to watch ~{estimated_directories} directories.\n\
                         To increase: sudo sysctl fs.inotify.max_user_watches=524288\n\
                         To make permanent: echo 'fs.inotify.max_user_watches=524288' | sudo tee -a /etc/sysctl.conf"
                    )),
                };
            }

            // Warn if approaching limit (> 80%)
            if needed > (max_watches as f64 * 0.8) as u64 {
                return PlatformLimits {
                    current_limit: Some(max_watches),
                    may_be_insufficient: false,
                    warning: Some(format!(
                        "Note: Approaching Linux inotify limit (using ~{needed}/{max_watches} watches).\n\
                         Consider increasing: sudo sysctl fs.inotify.max_user_watches=524288"
                    )),
                };
            }

            return PlatformLimits {
                current_limit: Some(max_watches),
                may_be_insufficient: false,
                warning: None,
            };
        }

        // Couldn't read limit, assume it's fine
        PlatformLimits {
            current_limit: None,
            may_be_insufficient: false,
            warning: None,
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = estimated_directories;
        PlatformLimits {
            current_limit: None,
            may_be_insufficient: false,
            warning: None,
        }
    }
}

#[cfg(target_os = "linux")]
fn read_inotify_max_user_watches() -> Option<u64> {
    std::fs::read_to_string("/proc/sys/fs/inotify/max_user_watches")
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// Estimate the number of directories in a path (for inotify limit check).
/// This does a quick walk without reading file contents.
pub fn estimate_directory_count(path: &Path) -> io::Result<usize> {
    let mut count = 0;
    count_directories_recursive(path, &mut count, 0, 10)?;
    Ok(count)
}

fn count_directories_recursive(
    path: &Path,
    count: &mut usize,
    depth: usize,
    max_depth: usize,
) -> io::Result<()> {
    if depth > max_depth {
        // Estimate: assume similar structure continues
        *count += 10;
        return Ok(());
    }

    if path.is_dir() {
        *count += 1;

        // Skip common large/unimportant directories
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if matches!(
            name,
            "node_modules" | ".git" | "target" | "build" | "dist" | ".cache"
        ) {
            // Estimate for these large dirs
            *count += 50;
            return Ok(());
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                count_directories_recursive(&entry_path, count, depth + 1, max_depth)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_inotify_limit_small_count() {
        let result = check_inotify_limit(10);
        // Should not warn for small counts
        assert!(!result.may_be_insufficient);
    }

    #[test]
    fn test_estimate_directory_count() {
        // Test with temp dir
        let result = estimate_directory_count(Path::new("."));
        assert!(result.is_ok());
    }
}
