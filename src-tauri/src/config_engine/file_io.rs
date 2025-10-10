// Atomic file I/O operations
// Ensures safe reading and writing of configuration files

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use tracing::{info, warn};

/// Read a file atomically
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    info!("Reading file: {:?}", path);

    fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {:?}", path))
}

/// Write a file atomically using temp file + rename
/// This ensures the original file is not corrupted if the write fails
pub fn write_file_atomic<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let path = path.as_ref();
    info!("Writing file atomically: {:?}", path);

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create parent directory")?;
    }

    // Write to temporary file in the same directory
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp_file = NamedTempFile::new_in(parent)
        .context("Failed to create temporary file")?;

    // Write content
    use std::io::Write;
    temp_file.write_all(content.as_bytes())
        .context("Failed to write to temporary file")?;

    temp_file.flush()
        .context("Failed to flush temporary file")?;

    // Atomically rename temp file to target
    temp_file.persist(path)
        .with_context(|| format!("Failed to persist file to {:?}", path))?;

    info!("File written successfully: {:?}", path);
    Ok(())
}

/// Check if a file exists
pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// Get file modification time as ISO 8601 string
pub fn get_file_modified_time<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to get metadata for {:?}", path))?;

    let modified = metadata.modified()
        .context("Failed to get modification time")?;

    let datetime: chrono::DateTime<chrono::Utc> = modified.into();
    Ok(datetime.to_rfc3339())
}

/// Backup a file by copying it with a timestamp suffix
pub fn backup_file<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();

    if !file_exists(path) {
        warn!("File does not exist, skipping backup: {:?}", path);
        return Ok(path.to_path_buf());
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = path.with_extension(format!("backup_{}.yml", timestamp));

    fs::copy(path, &backup_path)
        .with_context(|| format!("Failed to backup file {:?} to {:?}", path, backup_path))?;

    info!("File backed up: {:?} -> {:?}", path, backup_path);
    Ok(backup_path)
}

/// Delete a file
pub fn delete_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if !file_exists(path) {
        warn!("File does not exist, skipping delete: {:?}", path);
        return Ok(());
    }

    fs::remove_file(path)
        .with_context(|| format!("Failed to delete file: {:?}", path))?;

    info!("File deleted: {:?}", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_atomic_write_read() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let content = "Hello, World!";
        write_file_atomic(&file_path, content).unwrap();

        let read_content = read_file(&file_path).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_file_exists() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        assert!(!file_exists(&file_path));

        write_file_atomic(&file_path, "test").unwrap();
        assert!(file_exists(&file_path));
    }

    #[test]
    fn test_backup_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yml");

        write_file_atomic(&file_path, "original content").unwrap();

        let backup_path = backup_file(&file_path).unwrap();
        assert!(file_exists(&backup_path));

        let backup_content = read_file(&backup_path).unwrap();
        assert_eq!(backup_content, "original content");
    }

    #[test]
    fn test_delete_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        write_file_atomic(&file_path, "test").unwrap();
        assert!(file_exists(&file_path));

        delete_file(&file_path).unwrap();
        assert!(!file_exists(&file_path));
    }
}
