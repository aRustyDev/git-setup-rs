//! Atomic File Write Example
//! 
//! This example demonstrates how to safely write files atomically,
//! ensuring that files are either completely written or not modified at all.
//! This is critical for configuration files to prevent corruption.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Errors that can occur during atomic writes
#[derive(Debug, thiserror::Error)]
pub enum AtomicWriteError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    #[error("Permission denied")]
    PermissionDenied,
}

/// Atomically write data to a file
/// 
/// This ensures that the file is either completely written or not modified at all.
/// If the process crashes or fails during write, the original file remains intact.
pub fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), AtomicWriteError> {
    // Step 1: Get the parent directory
    let parent = path.parent()
        .ok_or_else(|| AtomicWriteError::InvalidPath("No parent directory".into()))?;
    
    // Step 2: Create a temporary file in the same directory
    // This ensures we can atomically rename it later
    let temp_file = NamedTempFile::new_in(parent)?;
    
    // Step 3: Write contents to temp file
    write_to_temp(&temp_file, contents)?;
    
    // Step 4: Set correct permissions (Unix only)
    #[cfg(unix)]
    set_secure_permissions(temp_file.path())?;
    
    // Step 5: Atomically rename temp file to target
    // This is the atomic operation - it either succeeds completely or fails
    temp_file.persist(path)?;
    
    Ok(())
}

/// Write contents to temporary file with proper sync
fn write_to_temp(temp_file: &NamedTempFile, contents: &[u8]) -> io::Result<()> {
    let mut file = temp_file.as_file();
    
    // Write all contents
    file.write_all(contents)?;
    
    // CRITICAL: Sync to disk before rename
    // This ensures data is actually on disk, not just in OS buffers
    file.sync_all()?;
    
    Ok(())
}

/// Set secure permissions on Unix systems
#[cfg(unix)]
fn set_secure_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    
    let mut perms = fs::metadata(path)?.permissions();
    // 0o600 = read/write for owner only
    perms.set_mode(0o600);
    fs::set_permissions(path, perms)?;
    
    Ok(())
}

/// Atomically write with automatic backup
pub fn atomic_write_with_backup(
    path: &Path,
    contents: &[u8],
) -> Result<PathBuf, AtomicWriteError> {
    // Create backup path
    let backup_path = create_backup_path(path);
    
    // If original file exists, create backup
    if path.exists() {
        fs::copy(path, &backup_path)?;
    }
    
    // Perform atomic write
    match atomic_write(path, contents) {
        Ok(()) => Ok(backup_path),
        Err(e) => {
            // On error, try to restore backup
            if backup_path.exists() {
                let _ = fs::copy(&backup_path, path);
            }
            Err(e)
        }
    }
}

/// Create backup filename
fn create_backup_path(path: &Path) -> PathBuf {
    let mut backup = path.to_path_buf();
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    backup.set_file_name(format!("{}.backup", filename));
    backup
}

/// Example usage and tests
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_atomic_write_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // Write file atomically
        atomic_write(&file_path, b"Hello, World!").unwrap();
        
        // Verify contents
        let contents = fs::read_to_string(&file_path).unwrap();
        assert_eq!(contents, "Hello, World!");
    }
    
    #[test]
    fn test_atomic_write_with_backup() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.toml");
        
        // Write initial content
        fs::write(&file_path, b"version = 1").unwrap();
        
        // Update with backup
        let backup = atomic_write_with_backup(&file_path, b"version = 2").unwrap();
        
        // Check new content
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "version = 2");
        
        // Check backup exists
        assert_eq!(fs::read_to_string(&backup).unwrap(), "version = 1");
    }
    
    #[test]
    #[cfg(unix)]
    fn test_permissions_preserved() {
        use std::os::unix::fs::PermissionsExt;
        
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("secure.txt");
        
        // Create file with specific permissions
        fs::write(&file_path, b"original").unwrap();
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&file_path, perms).unwrap();
        
        // Atomic write
        atomic_write(&file_path, b"updated").unwrap();
        
        // Check permissions (should be secure now)
        let new_perms = fs::metadata(&file_path).unwrap().permissions();
        assert_eq!(new_perms.mode() & 0o777, 0o600);
    }
}

/// Example program demonstrating atomic writes
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Atomic Write Example\n");
    
    // Example 1: Simple atomic write
    println!("1. Simple atomic write:");
    let config_path = Path::new("example_config.toml");
    let config_content = br#"
[profile]
name = "example"
email = "user@example.com"
"#;
    
    match atomic_write(config_path, config_content) {
        Ok(()) => println!("   ✓ Config written atomically"),
        Err(e) => println!("   ✗ Error: {}", e),
    }
    
    // Example 2: Atomic write with backup
    println!("\n2. Atomic write with backup:");
    match atomic_write_with_backup(config_path, b"updated content") {
        Ok(backup) => {
            println!("   ✓ File updated");
            println!("   ✓ Backup saved to: {:?}", backup);
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }
    
    // Cleanup
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_file("example_config.toml.backup");
    
    Ok(())
}

/// Junior Developer Notes:
/// 
/// 1. **Why Atomic Writes Matter**:
///    - Power failure during write = corrupted file
///    - Atomic write = all or nothing
///    - Critical for config files, databases, etc.
/// 
/// 2. **The Atomic Pattern**:
///    ```
///    1. Write to temp file
///    2. Sync to disk (fsync)
///    3. Rename temp → target (atomic!)
///    ```
/// 
/// 3. **Platform Differences**:
///    - Unix: rename() is atomic
///    - Windows: may need special handling
///    - Always test on target platforms
/// 
/// 4. **Common Mistakes**:
///    - Forgetting fsync() before rename
///    - Creating temp file in /tmp (different filesystem)
///    - Not handling permissions correctly
/// 
/// Exercise: Add these features:
/// 1. Multiple backup versions (config.toml.1, .2, etc)
/// 2. Compression for backups
/// 3. Verify file contents after write