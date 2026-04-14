// crabjar/agent-core/filesystem/src/filesystem.rs

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// File system trait for virtual filesystem abstraction
pub trait FileSystem {
    /// Read file content
    fn read(&self, path: &Path) -> std::io::Result<String>;

    /// Write content to file
    fn write(&mut self, path: &Path, content: &str) -> std::io::Result<()>;

    /// Check if path exists
    fn exists(&self, path: &Path) -> bool;

    /// Remove a file or directory
    fn remove(&mut self, path: &Path) -> std::io::Result<()>;

    /// Get file/directory statistics
    fn stat(&self, path: &Path) -> std::io::Result<FileStat>;

    /// Create a directory
    fn create_dir(&mut self, path: &Path) -> std::io::Result<()>;

    /// Create all parent directories if they don't exist
    fn create_dir_all(&mut self, path: &Path) -> std::io::Result<()>;

    /// Remove a directory (if empty)
    fn remove_dir(&mut self, path: &Path) -> std::io::Result<()>;

    /// Remove a directory recursively
    fn remove_dir_all(&mut self, path: &Path) -> std::io::Result<()>;

    /// List directory contents
    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<DirEntry>>;

    /// Create a new file
    fn create_file(&mut self, path: &Path) -> std::io::Result<()>;

    /// Copy a file or directory
    fn copy(&mut self, src: &Path, dst: &Path) -> std::io::Result<()>;

    /// Move a file or directory
    fn rename(&mut self, src: &Path, dst: &Path) -> std::io::Result<()>;

    /// Create a symbolic link
    fn symlink(&mut self, src: &Path, dst: &Path) -> std::io::Result<()>;

    /// Create a hard link
    fn link(&mut self, src: &Path, dst: &Path) -> std::io::Result<()>;

    /// Truncate a file
    fn truncate(&mut self, path: &Path, len: u64) -> std::io::Result<()>;

    /// Resize a file
    fn resize(&mut self, path: &Path, new_size: u64) -> std::io::Result<()>;

    /// Flush any buffered data
    fn flush(&mut self) -> std::io::Result<()>;
}

/// File system implementation that stores data in memory
pub struct InMemoryFs {
    root: HashMap<PathBuf, Vec<u8>>,
    symlinks: HashMap<PathBuf, PathBuf>,
}

impl InMemoryFs {
    pub fn new() -> Self {
        Self {
            root: HashMap::new(),
            symlinks: HashMap::new(),
        }
    }

    pub fn with_files(files: HashMap<PathBuf, Vec<u8>>) -> Self {
        Self {
            root: files,
            symlinks: HashMap::new(),
        }
    }
}

impl FileSystem for InMemoryFs {
    fn read(&self, path: &Path) -> std::io::Result<String> {
        if let Some(content) = self.root.get(path) {
            Ok(String::from_utf8_lossy(content).to_string())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No such file or directory: {:?}", path),
            ))
        }
    }

    fn write(&mut self, path: &Path, content: &str) -> std::io::Result<()> {
        self.root
            .insert(path.to_path_buf(), content.as_bytes().to_vec());
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        self.root.contains_key(path) || self.symlinks.contains_key(path)
    }

    fn remove(&mut self, path: &Path) -> std::io::Result<()> {
        if self.symlinks.remove(path).is_some() {
            return Ok(());
        }

        if self.root.remove(path).is_some() {
            return Ok(());
        }

        // Try to remove directory contents
        if let Some(entries) = self
            .root
            .iter()
            .filter(|(p, _)| p.starts_with(path))
            .collect::<Vec<_>>()
        {
            for (entry_path, _) in entries {
                let relative = entry_path.strip_prefix(path).unwrap();
                if relative.as_os_str().is_empty() {
                    continue;
                }
                self.remove(&path.join(relative))?;
            }
        }

        Ok(())
    }

    fn stat(&self, path: &Path) -> std::io::Result<FileStat> {
        if self.symlinks.contains_key(path) {
            return Ok(FileStat {
                is_dir: false,
                is_symlink: true,
                size: 0,
                modified: SystemTime::now(),
                permissions: 0o644,
            });
        }

        if let Some(content) = self.root.get(path) {
            return Ok(FileStat {
                is_dir: false,
                is_symlink: false,
                size: content.len() as u64,
                modified: SystemTime::now(),
                permissions: 0o644,
            });
        }

        // Check if it's a directory
        if let Some(entries) = self
            .root
            .iter()
            .filter(|(p, _)| p.starts_with(path))
            .collect::<Vec<_>>()
        {
            return Ok(FileStat {
                is_dir: true,
                is_symlink: false,
                size: entries.len() as u64,
                modified: SystemTime::now(),
                permissions: 0o755,
            });
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No such file or directory: {:?}", path),
        ))
    }

    fn create_dir(&mut self, path: &Path) -> std::io::Result<()> {
        if !path.as_os_str().is_empty() {
            self.root.insert(path.to_path_buf(), vec![]);
        }
        Ok(())
    }

    fn create_dir_all(&mut self, path: &Path) -> std::io::Result<()> {
        if path.as_os_str().is_empty() {
            return Ok(());
        }

        if self.exists(path) {
            return Ok(());
        }

        let parent = path
            .parent()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;

        self.create_dir_all(parent)?;
        self.create_dir(path)
    }

    fn remove_dir(&mut self, path: &Path) -> std::io::Result<()> {
        if !self.exists(path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No such directory: {:?}", path),
            ));
        }

        // Check if directory is empty
        if let Some(entries) = self
            .root
            .iter()
            .filter(|(p, _)| p.starts_with(path))
            .collect::<Vec<_>>()
        {
            let has_content = entries.iter().any(|(p, _)| {
                let relative = p.strip_prefix(path).unwrap();
                !relative.as_os_str().is_empty()
            });

            if has_content {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotEmpty,
                    "Directory not empty",
                ));
            }
        }

        self.remove(path)
    }

    fn remove_dir_all(&mut self, path: &Path) -> std::io::Result<()> {
        if !self.exists(path) {
            return Ok(());
        }

        // Recursively remove all contents
        if let Some(entries) = self
            .root
            .iter()
            .filter(|(p, _)| p.starts_with(path))
            .collect::<Vec<_>>()
        {
            for (entry_path, _) in entries {
                let relative = entry_path.strip_prefix(path).unwrap();
                if relative.as_os_str().is_empty() {
                    continue;
                }
                self.remove_dir_all(&path.join(relative))?;
            }
        }

        self.remove(path)
    }

    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<DirEntry>> {
        if !self.exists(path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No such directory: {:?}", path),
            ));
        }

        let mut entries = Vec::new();

        if let Some(entries) = self
            .root
            .iter()
            .filter(|(p, _)| p.starts_with(path))
            .collect::<Vec<_>>()
        {
            for (entry_path, _) in entries {
                let relative = entry_path.strip_prefix(path).unwrap();
                if relative.as_os_str().is_empty() {
                    continue;
                }

                let name = relative.file_name().unwrap().to_str().unwrap_or("");

                // Check if it's a directory
                let is_dir = self.root.contains_key(entry_path)
                    || self.root.iter().any(|(p, _)| p.starts_with(entry_path));

                entries.push(DirEntry {
                    name: name.to_string(),
                    path: entry_path.to_path_buf(),
                    is_dir,
                });
            }
        }

        // Add symlinks
        for (entry_path, target) in self.symlinks.iter() {
            if entry_path.starts_with(path) {
                let relative = entry_path.strip_prefix(path).unwrap();
                if !relative.as_os_str().is_empty() {
                    entries.push(DirEntry {
                        name: relative
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap_or("")
                            .to_string(),
                        path: entry_path.clone(),
                        is_dir: false,
                    });
                }
            }
        }

        Ok(entries)
    }

    fn create_file(&mut self, path: &Path) -> std::io::Result<()> {
        self.write(path, "")
    }

    fn copy(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        if !self.exists(src) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No such file or directory: {:?}", src),
            ));
        }

        let content = self.read(src)?;

        // Handle directories
        if let Some(entries) = self
            .root
            .iter()
            .filter(|(p, _)| p.starts_with(src))
            .collect::<Vec<_>>()
        {
            let has_content = entries.iter().any(|(p, _)| {
                let relative = p.strip_prefix(src).unwrap();
                !relative.as_os_str().is_empty()
            });

            if has_content {
                self.create_dir_all(dst)?;
                for (entry_path, _) in entries {
                    let relative = entry_path.strip_prefix(src).unwrap();
                    let dst_path = dst.join(relative);
                    self.copy(entry_path, &dst_path)?;
                }
                return Ok(());
            }
        }

        self.write(dst, &content)
    }

    fn rename(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        if !self.exists(src) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No such file or directory: {:?}", src),
            ));
        }

        // Handle directories
        if let Some(entries) = self
            .root
            .iter()
            .filter(|(p, _)| p.starts_with(src))
            .collect::<Vec<_>>()
        {
            let has_content = entries.iter().any(|(p, _)| {
                let relative = p.strip_prefix(src).unwrap();
                !relative.as_os_str().is_empty()
            });

            if has_content {
                self.create_dir_all(dst)?;
                for (entry_path, _) in entries {
                    let relative = entry_path.strip_prefix(src).unwrap();
                    let dst_path = dst.join(relative);
                    self.rename(entry_path, &dst_path)?;
                }
                return Ok(());
            }
        }

        let content = self.read(src)?;
        self.remove(src)?;
        self.write(dst, &content)
    }

    fn symlink(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        self.symlinks.insert(dst.to_path_buf(), src.to_path_buf());
        Ok(())
    }

    fn link(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        if !self.exists(src) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No such file or directory: {:?}", src),
            ));
        }

        let content = self.read(src)?;
        self.write(dst, &content)
    }

    fn truncate(&mut self, path: &Path, len: u64) -> std::io::Result<()> {
        if let Some(content) = self.root.get_mut(path) {
            if content.len() as u64 <= len {
                return Ok(());
            }
            content.truncate(len as usize);
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No such file: {:?}", path),
            ))
        }
    }

    fn resize(&mut self, path: &Path, new_size: u64) -> std::io::Result<()> {
        if let Some(content) = self.root.get_mut(path) {
            if content.len() as u64 > new_size {
                content.truncate(new_size as usize);
            } else {
                content.resize(new_size as usize, 0);
            }
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No such file: {:?}", path),
            ))
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // No-op for in-memory filesystem
        Ok(())
    }
}

/// File system implementation that overlays an in-memory filesystem on top of a real filesystem
pub struct OverlayFs {
    base: Box<dyn FileSystem>,
    memory: InMemoryFs,
}

impl OverlayFs {
    pub fn new(base: Box<dyn FileSystem>) -> Self {
        Self {
            base,
            memory: InMemoryFs::new(),
        }
    }

    pub fn with_base_and_memory(base: Box<dyn FileSystem>, memory: InMemoryFs) -> Self {
        Self { base, memory }
    }

    /// Get the mount point (root of the overlay)
    pub fn get_mount_point(&self) -> PathBuf {
        PathBuf::from("/home/user/project")
    }
}

impl FileSystem for OverlayFs {
    fn read(&self, path: &Path) -> std::io::Result<String> {
        // Check memory first
        if self.memory.exists(path) {
            return self.memory.read(path);
        }

        // Fall back to base
        self.base.read(path)
    }

    fn write(&mut self, path: &Path, content: &str) -> std::io::Result<()> {
        // Always write to memory
        self.memory.write(path, content)
    }

    fn exists(&self, path: &Path) -> bool {
        self.memory.exists(path) || self.base.exists(path)
    }

    fn remove(&mut self, path: &Path) -> std::io::Result<()> {
        // Always remove from memory
        self.memory.remove(path)
    }

    fn stat(&self, path: &Path) -> std::io::Result<FileStat> {
        // Check memory first
        if self.memory.exists(path) {
            return self.memory.stat(path);
        }

        // Fall back to base
        self.base.stat(path)
    }

    fn create_dir(&mut self, path: &Path) -> std::io::Result<()> {
        self.memory.create_dir(path)
    }

    fn create_dir_all(&mut self, path: &Path) -> std::io::Result<()> {
        self.memory.create_dir_all(path)
    }

    fn remove_dir(&mut self, path: &Path) -> std::io::Result<()> {
        self.memory.remove_dir(path)
    }

    fn remove_dir_all(&mut self, path: &Path) -> std::io::Result<()> {
        self.memory.remove_dir_all(path)
    }

    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<DirEntry>> {
        let mut entries = self.memory.read_dir(path)?;

        // Add entries from base that aren't in memory
        for entry in self.base.read_dir(path)? {
            if !entries.iter().any(|e| e.path == entry.path) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn create_file(&mut self, path: &Path) -> std::io::Result<()> {
        self.memory.create_file(path)
    }

    fn copy(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        // Copy to memory
        self.memory.copy(src, dst)
    }

    fn rename(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        // Rename in memory
        self.memory.rename(src, dst)
    }

    fn symlink(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        self.memory.symlink(src, dst)
    }

    fn link(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        self.memory.link(src, dst)
    }

    fn truncate(&mut self, path: &Path, len: u64) -> std::io::Result<()> {
        self.memory.truncate(path, len)
    }

    fn resize(&mut self, path: &Path, new_size: u64) -> std::io::Result<()> {
        self.memory.resize(path, new_size)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.memory.flush()
    }
}

/// File system implementation that provides direct read-write access to a real filesystem
pub struct ReadWriteFs {
    root: PathBuf,
}

impl ReadWriteFs {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl FileSystem for ReadWriteFs {
    fn read(&self, path: &Path) -> std::io::Result<String> {
        let full_path = self.root.join(path);
        std::fs::read_to_string(&full_path)
    }

    fn write(&mut self, path: &Path, content: &str) -> std::io::Result<()> {
        let full_path = self.root.join(path);
        std::fs::write(&full_path, content)
    }

    fn exists(&self, path: &Path) -> bool {
        let full_path = self.root.join(path);
        full_path.exists()
    }

    fn remove(&mut self, path: &Path) -> std::io::Result<()> {
        let full_path = self.root.join(path);
        std::fs::remove_file(&full_path)
    }

    fn stat(&self, path: &Path) -> std::io::Result<FileStat> {
        let full_path = self.root.join(path);
        let metadata = std::fs::metadata(&full_path)?;

        Ok(FileStat {
            is_dir: metadata.is_dir(),
            is_symlink: metadata.is_symlink(),
            size: metadata.len(),
            modified: metadata.modified()?,
            permissions: metadata.permissions().mode() as u32,
        })
    }

    fn create_dir(&mut self, path: &Path) -> std::io::Result<()> {
        let full_path = self.root.join(path);
        std::fs::create_dir(&full_path)
    }

    fn create_dir_all(&mut self, path: &Path) -> std::io::Result<()> {
        let full_path = self.root.join(path);
        std::fs::create_dir_all(&full_path)
    }

    fn remove_dir(&mut self, path: &Path) -> std::io::Result<()> {
        let full_path = self.root.join(path);
        std::fs::remove_dir(&full_path)
    }

    fn remove_dir_all(&mut self, path: &Path) -> std::io::Result<()> {
        let full_path = self.root.join(path);
        std::fs::remove_dir_all(&full_path)
    }

    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<DirEntry>> {
        let full_path = self.root.join(path);
        let entries = std::fs::read_dir(&full_path)?;

        let mut result = Vec::new();
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();

            result.push(DirEntry {
                name: name.to_string_lossy().to_string(),
                path: path,
                is_dir: entry.file_type().map(|t| t.is_dir()).unwrap_or(false),
            });
        }

        Ok(result)
    }

    fn create_file(&mut self, path: &Path) -> std::io::Result<()> {
        let full_path = self.root.join(path);
        std::fs::File::create(&full_path)
    }

    fn copy(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        let src_path = self.root.join(src);
        let dst_path = self.root.join(dst);
        std::fs::copy(&src_path, &dst_path)
    }

    fn rename(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        let src_path = self.root.join(src);
        let dst_path = self.root.join(dst);
        std::fs::rename(&src_path, &dst_path)
    }

    fn symlink(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        let src_path = self.root.join(src);
        let dst_path = self.root.join(dst);
        std::os::unix::fs::symlink(&src_path, &dst_path)
    }

    fn link(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        let src_path = self.root.join(src);
        let dst_path = self.root.join(dst);
        std::fs::hard_link(&src_path, &dst_path)
    }

    fn truncate(&mut self, path: &Path, len: u64) -> std::io::Result<()> {
        let full_path = self.root.join(path);
        let mut file = std::fs::OpenOptions::new().write(true).open(&full_path)?;
        file.set_len(len)
    }

    fn resize(&mut self, path: &Path, new_size: u64) -> std::io::Result<()> {
        let full_path = self.root.join(path);
        let mut file = std::fs::OpenOptions::new().write(true).open(&full_path)?;
        file.set_len(new_size)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // No-op for read-write filesystem
        Ok(())
    }
}

/// File system implementation that combines multiple filesystems at different mount points
pub struct MountableFs {
    base: Box<dyn FileSystem>,
    mounts: Vec<(PathBuf, Box<dyn FileSystem>)>,
}

impl MountableFs {
    pub fn new(base: Box<dyn FileSystem>) -> Self {
        Self {
            base,
            mounts: Vec::new(),
        }
    }

    pub fn mount(&mut self, mount_point: PathBuf, filesystem: Box<dyn FileSystem>) {
        self.mounts.push((mount_point, filesystem));
    }
}

impl FileSystem for MountableFs {
    fn read(&self, path: &Path) -> std::io::Result<String> {
        // Find the first mount that contains the path
        for (mount_point, fs) in &self.mounts {
            if path.starts_with(mount_point) {
                let relative = path.strip_prefix(mount_point).unwrap();
                return fs.read(relative);
            }
        }

        // Fall back to base
        self.base.read(path)
    }

    fn write(&mut self, path: &Path, content: &str) -> std::io::Result<()> {
        // Find the first mount that contains the path
        for (mount_point, fs) in &mut self.mounts {
            if path.starts_with(mount_point) {
                let relative = path.strip_prefix(mount_point).unwrap();
                return fs.write(relative, content);
            }
        }

        // Fall back to base
        self.base.write(path, content)
    }

    fn exists(&self, path: &Path) -> bool {
        // Find the first mount that contains the path
        for (mount_point, fs) in &self.mounts {
            if path.starts_with(mount_point) {
                let relative = path.strip_prefix(mount_point).unwrap();
                return fs.exists(relative);
            }
        }

        // Fall back to base
        self.base.exists(path)
    }

    fn remove(&mut self, path: &Path) -> std::io::Result<()> {
        // Find the first mount that contains the path
        for (mount_point, fs) in &mut self.mounts {
            if path.starts_with(mount_point) {
                let relative = path.strip_prefix(mount_point).unwrap();
                return fs.remove(relative);
            }
        }

        // Fall back to base
        self.base.remove(path)
    }

    fn stat(&self, path: &Path) -> std::io::Result<FileStat> {
        // Find the first mount that contains the path
        for (mount_point, fs) in &self.mounts {
            if path.starts_with(mount_point) {
                let relative = path.strip_prefix(mount_point).unwrap();
                return fs.stat(relative);
            }
        }

        // Fall back to base
        self.base.stat(path)
    }

    fn create_dir(&mut self, path: &Path) -> std::io::Result<()> {
        // Find the first mount that contains the path
        for (mount_point, fs) in &mut self.mounts {
            if path.starts_with(mount_point) {
                let relative = path.strip_prefix(mount_point).unwrap();
                return fs.create_dir(relative);
            }
        }

        // Fall back to base
        self.base.create_dir(path)
    }

    fn create_dir_all(&mut self, path: &Path) -> std::io::Result<()> {
        // Find the first mount that contains the path
        for (mount_point, fs) in &mut self.mounts {
            if path.starts_with(mount_point) {
                let relative = path.strip_prefix(mount_point).unwrap();
                return fs.create_dir_all(relative);
            }
        }

        // Fall back to base
        self.base.create_dir_all(path)
    }

    fn remove_dir(&mut self, path: &Path) -> std::io::Result<()> {
        // Find the first mount that contains the path
        for (mount_point, fs) in &mut self.mounts {
            if path.starts_with(mount_point) {
                let relative = path.strip_prefix(mount_point).unwrap();
                return fs.remove_dir(relative);
            }
        }

        // Fall back to base
        self.base.remove_dir(path)
    }

    fn remove_dir_all(&mut self, path: &Path) -> std::io::Result<()> {
        // Find the first mount that contains the path
        for (mount_point, fs) in &mut self.mounts {
            if path.starts_with(mount_point) {
                let relative = path.strip_prefix(mount_point).unwrap();
                return fs.remove_dir_all(relative);
            }
        }

        // Fall back to base
        self.base.remove_dir_all(path)
    }

    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<DirEntry>> {
        // Find the first mount that contains the path
        for (mount_point, fs) in &self.mounts {
            if path.starts_with(mount_point) {
                let relative = path.strip_prefix(mount_point).unwrap();
                return fs.read_dir(relative);
            }
        }

        // Fall back to base
        self.base.read_dir(path)
    }

    fn create_file(&mut self, path: &Path) -> std::io::Result<()> {
        // Find the first mount that contains the path
        for (mount_point, fs) in &mut self.mounts {
            if path.starts_with(mount_point) {
                let relative = path.strip_prefix(mount_point).unwrap();
                return fs.create_file(relative);
            }
        }

        // Fall back to base
        self.base.create_file(path)
    }

    fn copy(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        // Find the mount for source
        let source_mount = self
            .mounts
            .iter()
            .find(|(mount_point, _)| src.starts_with(mount_point));

        if let Some((mount_point, fs)) = source_mount {
            let relative_src = src.strip_prefix(mount_point).unwrap();

            // Find the mount for destination
            let dest_mount = self
                .mounts
                .iter()
                .find(|(mount_point, _)| dst.starts_with(mount_point));

            if let Some((dest_mount_point, dest_fs)) = dest_mount {
                let relative_dst = dst.strip_prefix(dest_mount_point).unwrap();
                return fs.copy(relative_src, relative_dst);
            }
        }

        // Fall back to base
        self.base.copy(src, dst)
    }

    fn rename(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        // Find the mount for source
        let source_mount = self
            .mounts
            .iter()
            .find(|(mount_point, _)| src.starts_with(mount_point));

        if let Some((mount_point, fs)) = source_mount {
            let relative_src = src.strip_prefix(mount_point).unwrap();

            // Find the mount for destination
            let dest_mount = self
                .mounts
                .iter()
                .find(|(mount_point, _)| dst.starts_with(mount_point));

            if let Some((dest_mount_point, dest_fs)) = dest_mount {
                let relative_dst = dst.strip_prefix(dest_mount_point).unwrap();
                return fs.rename(relative_src, relative_dst);
            }
        }

        // Fall back to base
        self.base.rename(src, dst)
    }

    fn symlink(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        // Find the mount for destination
        let dest_mount = self
            .mounts
            .iter()
            .find(|(mount_point, _)| dst.starts_with(mount_point));

        if let Some((mount_point, fs)) = dest_mount {
            let relative_dst = dst.strip_prefix(mount_point).unwrap();
            return fs.symlink(src, relative_dst);
        }

        // Fall back to base
        self.base.symlink(src, dst)
    }

    fn link(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        // Find the mount for destination
        let dest_mount = self
            .mounts
            .iter()
            .find(|(mount_point, _)| dst.starts_with(mount_point));

        if let Some((mount_point, fs)) = dest_mount {
            let relative_dst = dst.strip_prefix(mount_point).unwrap();
            return fs.link(src, relative_dst);
        }

        // Fall back to base
        self.base.link(src, dst)
    }

    fn truncate(&mut self, path: &Path, len: u64) -> std::io::Result<()> {
        // Find the mount that contains the path
        for (mount_point, fs) in &mut self.mounts {
            if path.starts_with(mount_point) {
                let relative = path.strip_prefix(mount_point).unwrap();
                return fs.truncate(relative, len);
            }
        }

        // Fall back to base
        self.base.truncate(path, len)
    }

    fn resize(&mut self, path: &Path, new_size: u64) -> std::io::Result<()> {
        // Find the mount that contains the path
        for (mount_point, fs) in &mut self.mounts {
            if path.starts_with(mount_point) {
                let relative = path.strip_prefix(mount_point).unwrap();
                return fs.resize(relative, new_size);
            }
        }

        // Fall back to base
        self.base.resize(path, new_size)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Flush all mounts
        for (_, fs) in &mut self.mounts {
            fs.flush()?;
        }
        self.base.flush()
    }
}

/// File statistics
#[derive(Debug, Clone)]
pub struct FileStat {
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: SystemTime,
    pub permissions: u32,
}

/// Directory entry
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_fs() {
        let mut fs = InMemoryFs::new();

        // Test file operations
        fs.write(Path::new("/test.txt"), "hello").unwrap();
        assert!(fs.exists(Path::new("/test.txt")));
        assert_eq!(fs.read(Path::new("/test.txt")).unwrap(), "hello");

        // Test directory operations
        fs.create_dir(Path::new("/dir")).unwrap();
        fs.write(Path::new("/dir/file.txt"), "world").unwrap();
        assert!(fs.exists(Path::new("/dir/file.txt")));

        // Test list directory
        let entries = fs.read_dir(Path::new("/dir")).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "file.txt");

        // Test remove
        fs.remove(Path::new("/test.txt")).unwrap();
        assert!(!fs.exists(Path::new("/test.txt")));
    }

    #[test]
    fn test_overlay_fs() {
        let base = Box::new(InMemoryFs::new());
        let mut overlay = OverlayFs::new(base);

        // Read from base
        overlay
            .write(Path::new("/base.txt"), "base content")
            .unwrap();
        assert_eq!(
            overlay.read(Path::new("/base.txt")).unwrap(),
            "base content"
        );

        // Write to overlay
        overlay
            .write(Path::new("/overlay.txt"), "overlay content")
            .unwrap();
        assert_eq!(
            overlay.read(Path::new("/overlay.txt")).unwrap(),
            "overlay content"
        );

        // Remove from overlay
        overlay.remove(Path::new("/overlay.txt")).unwrap();
        assert!(!overlay.exists(Path::new("/overlay.txt")));
    }

    #[test]
    fn test_mountable_fs() {
        let base = Box::new(InMemoryFs::new());
        let mut mountable = MountableFs::new(base);

        let memory_fs = Box::new(InMemoryFs::new());
        mountable.mount(PathBuf::from("/mnt"), memory_fs);

        // Write to mount
        mountable
            .write(Path::new("/mnt/file.txt"), "mounted")
            .unwrap();
        assert_eq!(
            mountable.read(Path::new("/mnt/file.txt")).unwrap(),
            "mounted"
        );

        // Write to base
        mountable.write(Path::new("/base.txt"), "base").unwrap();
        assert_eq!(mountable.read(Path::new("/base.txt")).unwrap(), "base");
    }
}
