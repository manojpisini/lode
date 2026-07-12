use std::path::{Path, PathBuf};

pub fn socket_port(socket_path: &Path) -> u16 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    let hash = {
        socket_path.to_string_lossy().hash(&mut hasher);
        hasher.finish()
    };
    42000 + ((hash % 20000) as u16)
}

pub fn port_path(socket_path: &Path) -> PathBuf {
    let mut p = socket_path.to_path_buf();
    p.set_extension("port");
    p
}
