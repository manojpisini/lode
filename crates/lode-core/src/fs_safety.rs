use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{LodeError, Result};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A canonical filesystem boundary for validated, root-relative operations.
#[derive(Debug, Clone)]
pub struct ValidatedRoot {
    root: PathBuf,
}

impl ValidatedRoot {
    /// Creates a boundary from an existing directory.
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let path = root.as_ref();
        let root = fs::canonicalize(path).map_err(|e| io_error(path, e))?;
        if !fs::metadata(&root)
            .map_err(|e| io_error(&root, e))?
            .is_dir()
        {
            return Err(unsafe_path(path, "root is not a directory"));
        }
        Ok(Self { root })
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    /// Resolves a relative descendant without allowing lexical or symlink escapes.
    pub fn resolve(&self, relative: impl AsRef<Path>) -> Result<PathBuf> {
        let relative = relative.as_ref();
        for component in relative.components() {
            match component {
                Component::Normal(_) | Component::CurDir => {}
                Component::ParentDir => {
                    return Err(unsafe_path(relative, "parent traversal is not allowed"))
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(unsafe_path(relative, "absolute paths are not allowed"))
                }
            }
        }
        let target = self.root.join(relative);
        let mut existing = target.as_path();
        loop {
            match fs::symlink_metadata(existing) {
                Ok(_) => {
                    self.ensure_within(existing)?;
                    return Ok(target);
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    existing = existing
                        .parent()
                        .ok_or_else(|| unsafe_path(&target, "no existing ancestor"))?;
                }
                Err(e) => return Err(io_error(existing, e)),
            }
        }
    }

    /// Creates a directory tree while revalidating every created component.
    pub fn create_dir_all(&self, relative: impl AsRef<Path>) -> Result<PathBuf> {
        let target = self.resolve(relative)?;
        let suffix = target
            .strip_prefix(&self.root)
            .expect("validated descendant");
        let mut current = self.root.clone();
        for component in suffix.components() {
            current.push(component.as_os_str());
            match fs::symlink_metadata(&current) {
                Ok(_) => {
                    self.ensure_within(&current)?;
                    if !fs::metadata(&current)
                        .map_err(|e| io_error(&current, e))?
                        .is_dir()
                    {
                        return Err(unsafe_path(&current, "component is not a directory"));
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    fs::create_dir(&current).map_err(|e| io_error(&current, e))?;
                    self.ensure_within(&current)?;
                }
                Err(e) => return Err(io_error(&current, e)),
            }
        }
        Ok(target)
    }

    /// Writes through a same-directory temporary file and atomically installs it.
    pub fn write_atomic(
        &self,
        relative: impl AsRef<Path>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<PathBuf> {
        let target = self.resolve(relative)?;
        if target == self.root {
            return Err(unsafe_path(&target, "cannot write the root"));
        }
        let parent = target
            .parent()
            .ok_or_else(|| unsafe_path(&target, "missing parent"))?;
        self.ensure_within(parent)?;
        if !fs::metadata(parent)
            .map_err(|e| io_error(parent, e))?
            .is_dir()
        {
            return Err(unsafe_path(parent, "parent is not a directory"));
        }
        let replacing = match fs::symlink_metadata(&target) {
            Ok(meta) if meta.is_file() && !meta.file_type().is_symlink() => true,
            Ok(_) => return Err(unsafe_path(&target, "only regular files can be replaced")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(e) => return Err(io_error(&target, e)),
        };
        let (temporary, mut file) = self.temporary(parent)?;
        let result: Result<()> = (|| {
            file.write_all(bytes.as_ref())
                .map_err(|e| io_error(&temporary, e))?;
            file.sync_all().map_err(|e| io_error(&temporary, e))?;
            drop(file);
            replace_file(&temporary, &target, replacing).map_err(|e| io_error(&target, e))?;
            sync_directory(parent)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result?;
        Ok(target)
    }

    pub fn remove_file(&self, relative: impl AsRef<Path>) -> Result<()> {
        let target = self.resolve(relative)?;
        if target == self.root {
            return Err(unsafe_path(&target, "cannot remove the root"));
        }
        let meta = fs::symlink_metadata(&target).map_err(|e| io_error(&target, e))?;
        if meta.file_type().is_symlink() || !meta.is_file() {
            return Err(unsafe_path(&target, "not a regular file"));
        }
        fs::remove_file(&target).map_err(|e| io_error(&target, e))
    }

    pub fn remove_empty_dir(&self, relative: impl AsRef<Path>) -> Result<()> {
        let target = self.resolve(relative)?;
        if target == self.root {
            return Err(unsafe_path(&target, "cannot remove the root"));
        }
        let meta = fs::symlink_metadata(&target).map_err(|e| io_error(&target, e))?;
        if meta.file_type().is_symlink() || !meta.is_dir() {
            return Err(unsafe_path(&target, "not a directory"));
        }
        fs::remove_dir(&target).map_err(|e| io_error(&target, e))
    }

    fn ensure_within(&self, path: &Path) -> Result<()> {
        let canonical = fs::canonicalize(path).map_err(|e| io_error(path, e))?;
        if canonical == self.root || canonical.starts_with(&self.root) {
            Ok(())
        } else {
            Err(unsafe_path(path, "symlink escapes the validated root"))
        }
    }

    fn temporary(&self, parent: &Path) -> Result<(PathBuf, fs::File)> {
        for _ in 0..128 {
            let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!(".lode-atomic-{}-{id}.tmp", std::process::id()));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => return Ok((path, file)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(io_error(&path, e)),
            }
        }
        Err(unsafe_path(parent, "unable to allocate temporary file"))
    }
}

fn unsafe_path(path: &Path, reason: &str) -> LodeError {
    LodeError::Message(format!(
        "unsafe filesystem path {}: {reason}",
        path.display()
    ))
}
fn io_error(path: &Path, source: std::io::Error) -> LodeError {
    LodeError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<()> {
    fs::File::open(path)
        .and_then(|f| f.sync_all())
        .map_err(|e| io_error(path, e))
}
#[cfg(not(unix))]
fn sync_directory(_: &Path) -> Result<()> {
    Ok(())
}

#[cfg(not(windows))]
fn replace_file(from: &Path, to: &Path, _: bool) -> std::io::Result<()> {
    fs::rename(from, to)
}

#[cfg(windows)]
fn replace_file(from: &Path, to: &Path, replacing: bool) -> std::io::Result<()> {
    if !replacing {
        return fs::rename(from, to);
    }
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn ReplaceFileW(
            replaced: *const u16,
            replacement: *const u16,
            backup: *const u16,
            flags: u32,
            exclude: *mut std::ffi::c_void,
            reserved: *mut std::ffi::c_void,
        ) -> i32;
    }
    let to: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    let from: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: Both path buffers are NUL-terminated and remain alive for the call.
    let result = unsafe {
        ReplaceFileW(
            to.as_ptr(),
            from.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if result == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, ValidatedRoot) {
        let temp = tempfile::tempdir().unwrap();
        let root = ValidatedRoot::new(temp.path()).unwrap();
        (temp, root)
    }

    #[test]
    fn root_must_exist_and_be_directory() {
        let temp = tempfile::tempdir().unwrap();
        assert!(ValidatedRoot::new(temp.path().join("missing")).is_err());
        let file = temp.path().join("file");
        fs::write(&file, b"x").unwrap();
        assert!(ValidatedRoot::new(file).is_err());
    }

    #[test]
    fn resolution_rejects_traversal_and_absolute_paths() {
        let (_temp, root) = setup();
        assert_eq!(root.resolve("a/./b").unwrap(), root.path().join("a/b"));
        assert!(root.resolve("../escape").is_err());
        assert!(root.resolve("a/../../escape").is_err());
        assert!(root.resolve(std::env::current_dir().unwrap()).is_err());
    }

    #[test]
    fn creates_nested_directories_and_rejects_file_components() {
        let (_temp, root) = setup();
        assert!(root.create_dir_all("a/b/c").unwrap().is_dir());
        fs::write(root.path().join("file"), b"x").unwrap();
        assert!(root.create_dir_all("file/child").is_err());
    }

    #[test]
    fn atomic_write_creates_and_replaces_without_temp_files() {
        let (_temp, root) = setup();
        root.create_dir_all("dir").unwrap();
        let target = root.write_atomic("dir/file", b"one").unwrap();
        root.write_atomic("dir/file", b"two").unwrap();
        assert_eq!(
            fs::read(target.parent().unwrap().join("file")).unwrap(),
            b"two"
        );
        assert_eq!(fs::read_dir(target.parent().unwrap()).unwrap().count(), 1);
    }

    #[test]
    fn atomic_write_rejects_missing_parent_and_non_file_target() {
        let (_temp, root) = setup();
        root.create_dir_all("dir").unwrap();
        assert!(root.write_atomic("missing/file", b"x").is_err());
        assert!(root.write_atomic("dir", b"x").is_err());
        assert!(root.write_atomic("", b"x").is_err());
    }

    #[test]
    fn removal_is_typed_and_directories_must_be_empty() {
        let (_temp, root) = setup();
        root.create_dir_all("dir").unwrap();
        root.write_atomic("dir/file", b"x").unwrap();
        assert!(root.remove_empty_dir("dir").is_err());
        assert!(root.remove_file("dir").is_err());
        assert!(root.remove_empty_dir("dir/file").is_err());
        root.remove_file("dir/file").unwrap();
        root.remove_empty_dir("dir").unwrap();
        assert!(root.remove_file("").is_err());
        assert!(root.remove_empty_dir("").is_err());
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn symlink_escape_is_rejected_by_all_operations() {
        let (_temp, root) = setup();
        let outside = tempfile::tempdir().unwrap();
        if !directory_link(outside.path(), &root.path().join("escape")) {
            return;
        }
        fs::write(outside.path().join("file"), b"x").unwrap();
        fs::create_dir(outside.path().join("empty")).unwrap();
        assert!(root.resolve("escape/file").is_err());
        assert!(root.create_dir_all("escape/new").is_err());
        assert!(root.write_atomic("escape/file", b"changed").is_err());
        assert!(root.remove_file("escape/file").is_err());
        assert!(root.remove_empty_dir("escape/empty").is_err());
        assert_eq!(fs::read(outside.path().join("file")).unwrap(), b"x");
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn internal_symlink_is_allowed_but_file_symlink_is_not_mutated() {
        let (_temp, root) = setup();
        root.create_dir_all("real").unwrap();
        if !directory_link(&root.path().join("real"), &root.path().join("link")) {
            return;
        }
        root.write_atomic("link/file", b"x").unwrap();
        if !file_link(
            &root.path().join("real/file"),
            &root.path().join("file-link"),
        ) {
            return;
        }
        assert!(root.write_atomic("file-link", b"y").is_err());
        assert!(root.remove_file("file-link").is_err());
        assert_eq!(fs::read(root.path().join("real/file")).unwrap(), b"x");
    }

    #[cfg(unix)]
    fn directory_link(target: &Path, link: &Path) -> bool {
        std::os::unix::fs::symlink(target, link).unwrap();
        true
    }
    #[cfg(unix)]
    fn file_link(target: &Path, link: &Path) -> bool {
        std::os::unix::fs::symlink(target, link).unwrap();
        true
    }
    #[cfg(windows)]
    fn directory_link(target: &Path, link: &Path) -> bool {
        std::os::windows::fs::symlink_dir(target, link).is_ok()
    }
    #[cfg(windows)]
    fn file_link(target: &Path, link: &Path) -> bool {
        std::os::windows::fs::symlink_file(target, link).is_ok()
    }
}
