use std::path::PathBuf;

/// Finder-launched applications do not normally inherit a shell's Homebrew PATH.
pub(crate) fn executable(name: &str) -> PathBuf {
    if let Some(path) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&path) {
            let path = directory.join(if cfg!(windows) {
                format!("{name}.exe")
            } else {
                name.into()
            });
            if path.is_file() {
                return path;
            }
        }
    }
    #[cfg(target_os = "macos")]
    for directory in ["/opt/homebrew/bin", "/usr/local/bin"] {
        let path = std::path::Path::new(directory).join(name);
        if path.is_file() {
            return path;
        }
    }
    name.into()
}
