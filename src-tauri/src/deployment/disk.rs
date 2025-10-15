// T175: Disk information and volume management
// Provides disk space information and volume validation

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub mount_point: PathBuf,
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f64,
    pub total_formatted: String,
    pub used_formatted: String,
    pub free_formatted: String,
    pub filesystem_type: Option<String>,
}

impl DiskInfo {
    /// Check if disk has low free space (< 10GB threshold)
    pub fn is_low_space(&self) -> bool {
        const LOW_SPACE_THRESHOLD: u64 = 10 * 1024 * 1024 * 1024; // 10 GB
        self.free < LOW_SPACE_THRESHOLD
    }

    /// Get human-readable size string
    pub fn format_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Get disk information for a given path
pub fn get_disk_info<P: AsRef<Path>>(path: P) -> Result<DiskInfo, String> {
    let path = path.as_ref();

    // Ensure path exists
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    info!("Getting disk info for path: {}", path.display());

    #[cfg(target_os = "linux")]
    {
        get_disk_info_linux(path)
    }

    #[cfg(target_os = "macos")]
    {
        get_disk_info_macos(path)
    }

    #[cfg(target_os = "windows")]
    {
        get_disk_info_windows(path)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err("Unsupported operating system".to_string())
    }
}

#[cfg(target_os = "linux")]
fn get_disk_info_linux(path: &Path) -> Result<DiskInfo, String> {
    use std::process::Command;

    // Use `df` command to get disk information
    let output = Command::new("df")
        .arg("-B1") // Output in bytes
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to execute df command: {}", e))?;

    if !output.status.success() {
        return Err("df command failed".to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();

    if lines.len() < 2 {
        return Err("Unexpected df output format".to_string());
    }

    // Parse df output (skip header line)
    let data_line = lines[1];
    let parts: Vec<&str> = data_line.split_whitespace().collect();

    if parts.len() < 6 {
        return Err("Unexpected df output format".to_string());
    }

    let filesystem = parts[0].to_string();
    let total = parts[1].parse::<u64>().map_err(|e| e.to_string())?;
    let used = parts[2].parse::<u64>().map_err(|e| e.to_string())?;
    let mount_point_str = parts[5];

    let free = total - used;
    let usage_percent = if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    Ok(DiskInfo {
        mount_point: PathBuf::from(mount_point_str),
        total,
        used,
        free,
        usage_percent,
        total_formatted: DiskInfo::format_size(total),
        used_formatted: DiskInfo::format_size(used),
        free_formatted: DiskInfo::format_size(free),
        filesystem_type: Some(filesystem),
    })
}

#[cfg(target_os = "macos")]
fn get_disk_info_macos(path: &Path) -> Result<DiskInfo, String> {
    use std::process::Command;

    // Use `df` command (macOS version)
    let output = Command::new("df")
        .arg("-k") // Output in kilobytes
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to execute df command: {}", e))?;

    if !output.status.success() {
        return Err("df command failed".to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();

    if lines.len() < 2 {
        return Err("Unexpected df output format".to_string());
    }

    let data_line = lines[1];
    let parts: Vec<&str> = data_line.split_whitespace().collect();

    if parts.len() < 9 {
        return Err("Unexpected df output format".to_string());
    }

    let filesystem = parts[0].to_string();
    let total_kb = parts[1].parse::<u64>().map_err(|e| e.to_string())?;
    let used_kb = parts[2].parse::<u64>().map_err(|e| e.to_string())?;
    let mount_point_str = parts[8];

    // Convert kilobytes to bytes
    let total = total_kb * 1024;
    let used = used_kb * 1024;
    let free = total - used;

    let usage_percent = if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    Ok(DiskInfo {
        mount_point: PathBuf::from(mount_point_str),
        total,
        used,
        free,
        usage_percent,
        total_formatted: DiskInfo::format_size(total),
        used_formatted: DiskInfo::format_size(used),
        free_formatted: DiskInfo::format_size(free),
        filesystem_type: Some(filesystem),
    })
}

#[cfg(target_os = "windows")]
fn get_disk_info_windows(path: &Path) -> Result<DiskInfo, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    // Get the root path (drive letter)
    let root = path
        .ancestors()
        .last()
        .ok_or_else(|| "Failed to get root path".to_string())?;

    // Convert path to wide string for Windows API
    let root_wide: Vec<u16> = OsStr::new(&root)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut free_bytes_available: u64 = 0;
    let mut total: u64 = 0;
    let mut free_total: u64 = 0;

    unsafe {
        use winapi::um::fileapi::GetDiskFreeSpaceExW;

        let result = GetDiskFreeSpaceExW(
            root_wide.as_ptr(),
            &mut free_bytes_available as *mut u64 as *mut _,
            &mut total as *mut u64 as *mut _,
            &mut free_total as *mut u64 as *mut _,
        );

        if result == 0 {
            return Err("Failed to get disk space information".to_string());
        }
    }

    let used = total - free_total;
    let free = free_total;
    let usage_percent = if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    Ok(DiskInfo {
        mount_point: root.to_path_buf(),
        total,
        used,
        free,
        usage_percent,
        total_formatted: DiskInfo::format_size(total),
        used_formatted: DiskInfo::format_size(used),
        free_formatted: DiskInfo::format_size(free),
        filesystem_type: Some("NTFS".to_string()), // Default assumption
    })
}

/// List all available disks/mount points
pub fn list_all_disks() -> Result<Vec<DiskInfo>, String> {
    info!("Listing all available disks");

    #[cfg(target_os = "linux")]
    {
        list_all_disks_linux()
    }

    #[cfg(target_os = "macos")]
    {
        list_all_disks_macos()
    }

    #[cfg(target_os = "windows")]
    {
        list_all_disks_windows()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err("Unsupported operating system".to_string())
    }
}

#[cfg(target_os = "linux")]
fn list_all_disks_linux() -> Result<Vec<DiskInfo>, String> {
    use std::process::Command;

    // Use `df` to list all mounted filesystems
    let output = Command::new("df")
        .arg("-B1") // Output in bytes
        .arg("-T") // Show filesystem type
        .output()
        .map_err(|e| format!("Failed to execute df command: {}", e))?;

    if !output.status.success() {
        return Err("df command failed".to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();

    if lines.is_empty() {
        return Err("No df output".to_string());
    }

    let mut disks = Vec::new();

    // Skip header line
    for line in lines.iter().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 7 {
            continue;
        }

        // Skip tmpfs, devtmpfs, and other virtual filesystems
        let fs_type = parts[1];
        if fs_type.starts_with("tmpfs")
            || fs_type.starts_with("devtmpfs")
            || fs_type == "overlay"
            || fs_type == "squashfs"
        {
            continue;
        }

        let filesystem = parts[0].to_string();
        let total = parts[2].parse::<u64>().unwrap_or(0);
        let used = parts[3].parse::<u64>().unwrap_or(0);
        let mount_point_str = parts[6];

        let free = total - used;
        let usage_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        disks.push(DiskInfo {
            mount_point: PathBuf::from(mount_point_str),
            total,
            used,
            free,
            usage_percent,
            total_formatted: DiskInfo::format_size(total),
            used_formatted: DiskInfo::format_size(used),
            free_formatted: DiskInfo::format_size(free),
            filesystem_type: Some(fs_type.to_string()),
        });
    }

    Ok(disks)
}

#[cfg(target_os = "macos")]
fn list_all_disks_macos() -> Result<Vec<DiskInfo>, String> {
    use std::process::Command;

    let output = Command::new("df")
        .arg("-k")
        .output()
        .map_err(|e| format!("Failed to execute df command: {}", e))?;

    if !output.status.success() {
        return Err("df command failed".to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();

    if lines.is_empty() {
        return Err("No df output".to_string());
    }

    let mut disks = Vec::new();

    for line in lines.iter().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 9 {
            continue;
        }

        let filesystem = parts[0].to_string();

        // Skip devfs and other virtual filesystems
        if filesystem.starts_with("devfs") || filesystem.starts_with("map") {
            continue;
        }

        let total_kb = parts[1].parse::<u64>().unwrap_or(0);
        let used_kb = parts[2].parse::<u64>().unwrap_or(0);
        let mount_point_str = parts[8];

        let total = total_kb * 1024;
        let used = used_kb * 1024;
        let free = total - used;

        let usage_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        disks.push(DiskInfo {
            mount_point: PathBuf::from(mount_point_str),
            total,
            used,
            free,
            usage_percent,
            total_formatted: DiskInfo::format_size(total),
            used_formatted: DiskInfo::format_size(used),
            free_formatted: DiskInfo::format_size(free),
            filesystem_type: Some(filesystem),
        });
    }

    Ok(disks)
}

#[cfg(target_os = "windows")]
fn list_all_disks_windows() -> Result<Vec<DiskInfo>, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let mut disks = Vec::new();

    // Check drives A-Z
    for drive_letter in b'A'..=b'Z' {
        let drive_path = format!("{}:\\", drive_letter as char);
        let path = std::path::Path::new(&drive_path);

        // Check if drive exists
        if !path.exists() {
            continue;
        }

        // Try to get disk info for this drive
        match get_disk_info(path) {
            Ok(disk_info) => disks.push(disk_info),
            Err(_) => continue, // Skip drives we can't read
        }
    }

    if disks.is_empty() {
        return Err("No drives found".to_string());
    }

    Ok(disks)
}

/// Check if disk space is below threshold and return warning message
pub fn check_disk_space_warning<P: AsRef<Path>>(
    path: P,
    threshold: u64,
) -> Result<Option<String>, String> {
    let disk_info = get_disk_info(path)?;

    if disk_info.free < threshold {
        let warning = format!(
            "Low disk space: {} free (< {} required) on {}",
            disk_info.free_formatted,
            DiskInfo::format_size(threshold),
            disk_info.mount_point.display()
        );
        Ok(Some(warning))
    } else {
        Ok(None)
    }
}

/// Validate if a path is suitable for volume mapping
pub fn validate_volume_path<P: AsRef<Path>>(path: P) -> Result<(), Vec<String>> {
    let path = path.as_ref();
    let mut errors = Vec::new();

    // Check if path exists
    if !path.exists() {
        errors.push(format!("Path does not exist: {}", path.display()));
        return Err(errors);
    }

    // Check if path is a directory
    if !path.is_dir() {
        errors.push(format!("Path is not a directory: {}", path.display()));
    }

    // Check if path is writable
    let test_file = path.join(".write_test_frigate");
    match std::fs::write(&test_file, "test") {
        Ok(_) => {
            // Successfully wrote - clean up
            let _ = std::fs::remove_file(&test_file);
        }
        Err(e) => {
            errors.push(format!("Path is not writable: {} ({})", path.display(), e));
        }
    }

    // Check disk space
    match get_disk_info(path) {
        Ok(disk_info) => {
            if disk_info.is_low_space() {
                warn!(
                    "Low disk space on {}: {} free",
                    path.display(),
                    disk_info.free_formatted
                );
                errors.push(format!(
                    "Warning: Low disk space (< 10GB) on {}",
                    path.display()
                ));
            }
        }
        Err(e) => {
            warn!("Could not get disk info for {}: {}", path.display(), e);
            // Don't fail validation just because we can't get disk info
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_disk_info_for_current_dir() {
        let current_dir = std::env::current_dir().unwrap();
        let disk_info = get_disk_info(&current_dir);

        assert!(disk_info.is_ok());

        let info = disk_info.unwrap();
        assert!(info.total > 0);
        assert!(info.used > 0);
        assert!(info.free > 0);
        assert!(info.usage_percent >= 0.0 && info.usage_percent <= 100.0);

        println!("Disk info for current directory:");
        println!("  Total: {}", info.total_formatted);
        println!("  Used: {}", info.used_formatted);
        println!("  Free: {}", info.free_formatted);
        println!("  Usage: {:.2}%", info.usage_percent);
        println!("  Mount point: {}", info.mount_point.display());
        println!("  Filesystem: {:?}", info.filesystem_type);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(DiskInfo::format_size(0), "0.00 B");
        assert_eq!(DiskInfo::format_size(1023), "1023.00 B");
        assert_eq!(DiskInfo::format_size(1024), "1.00 KB");
        assert_eq!(DiskInfo::format_size(1024 * 1024), "1.00 MB");
        assert_eq!(DiskInfo::format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(
            DiskInfo::format_size(1024u64 * 1024 * 1024 * 1024),
            "1.00 TB"
        );
    }

    #[test]
    fn test_is_low_space() {
        let mut disk_info = DiskInfo {
            mount_point: PathBuf::from("/"),
            total: 100 * 1024 * 1024 * 1024, // 100 GB
            used: 95 * 1024 * 1024 * 1024,   // 95 GB
            free: 5 * 1024 * 1024 * 1024,    // 5 GB
            usage_percent: 95.0,
            total_formatted: "100.00 GB".to_string(),
            used_formatted: "95.00 GB".to_string(),
            free_formatted: "5.00 GB".to_string(),
            filesystem_type: Some("ext4".to_string()),
        };

        // 5 GB is less than 10 GB threshold
        assert!(disk_info.is_low_space());

        // 15 GB is more than 10 GB threshold
        disk_info.free = 15 * 1024 * 1024 * 1024;
        assert!(!disk_info.is_low_space());
    }

    #[test]
    fn test_validate_volume_path_valid() {
        let temp_dir = TempDir::new().unwrap();
        let result = validate_volume_path(temp_dir.path());

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_volume_path_nonexistent() {
        let result = validate_volume_path("/nonexistent/path/that/does/not/exist");

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        assert!(errors[0].contains("does not exist"));
    }

    #[test]
    fn test_validate_volume_path_file_not_dir() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test").unwrap();

        let result = validate_volume_path(&file_path);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("not a directory")));
    }
}
