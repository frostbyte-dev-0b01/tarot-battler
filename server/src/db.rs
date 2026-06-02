//! Embedded database (redb) handle.
//!
//! redb is a pure-Rust embedded key/value store — chosen over SQLite so the
//! server builds with no C toolchain. Tables are created lazily by the data
//! layer; this scaffold just opens (or creates) the database file.

use std::path::Path;

use redb::Database;

pub struct Db {
    #[allow(dead_code)]
    pub inner: Database,
}

impl Db {
    /// Open the database at `path`, creating the file and any parent directories
    /// if they don't exist.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("create data dir {}: {e}", parent.display()))?;
            }
        }
        let inner = Database::create(path).map_err(|e| format!("open redb {}: {e}", path.display()))?;
        Ok(Self { inner })
    }
}
