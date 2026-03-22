//! fudajiku — BLAKE3 content-addressed manifest tracking.
//!
//! Tracks file state for incremental sync, backup, and deployment.
//! Stores BLAKE3 hashes, sizes, and timestamps in a JSON manifest.
//! Determines which files need updating by comparing hashes.
//!
//! Used by: andro-sync (file transfer), nexus (asset tracking),
//! blackmatter-profiles (image layer tracking).

use blake3::Hash;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FudajikuError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, FudajikuError>;

/// A single entry in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// The key (typically a remote path or file identifier).
    pub key: String,

    /// Local path where the file was synced to/from.
    pub local_path: String,

    /// BLAKE3 hash of the content.
    pub blake3_hash: String,

    /// File size in bytes.
    pub size: u64,

    /// When the content was last modified.
    pub modified: DateTime<Utc>,

    /// When this entry was last synced.
    pub synced_at: DateTime<Utc>,
}

/// Content-addressed manifest for tracking sync state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Map of key → entry.
    pub entries: HashMap<String, ManifestEntry>,

    /// When the last sync operation completed.
    pub last_sync: Option<DateTime<Utc>>,
}

impl Manifest {
    /// Create an empty manifest.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            last_sync: None,
        }
    }

    /// Load a manifest from a JSON file, or create empty if not found.
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(Self::new)
    }

    /// Save the manifest to a JSON file.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Record a file entry with its BLAKE3 hash.
    pub fn record(&mut self, key: &str, local_path: &str, hash: Hash, size: u64) {
        let now = Utc::now();
        self.entries.insert(
            key.to_string(),
            ManifestEntry {
                key: key.to_string(),
                local_path: local_path.to_string(),
                blake3_hash: hash.to_hex().to_string(),
                size,
                modified: now,
                synced_at: now,
            },
        );
        self.last_sync = Some(now);
    }

    /// Check if a file needs syncing by comparing BLAKE3 hashes.
    /// Returns true if the file is new or its hash has changed.
    pub fn needs_sync(&self, key: &str, current_hash: &str) -> bool {
        match self.entries.get(key) {
            Some(entry) => entry.blake3_hash != current_hash,
            None => true,
        }
    }

    /// Remove an entry from the manifest.
    pub fn remove(&mut self, key: &str) -> Option<ManifestEntry> {
        self.entries.remove(key)
    }

    /// List all keys in the manifest.
    pub fn keys(&self) -> Vec<&str> {
        self.entries.keys().map(String::as_str).collect()
    }

    /// Get the total size of all tracked entries.
    pub fn total_size(&self) -> u64 {
        self.entries.values().map(|e| e.size).sum()
    }

    /// Get entry count.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if manifest is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash file contents with BLAKE3.
pub fn hash_file(path: &Path) -> Result<Hash> {
    let data = std::fs::read(path)?;
    Ok(blake3::hash(&data))
}

/// Hash a byte slice with BLAKE3.
pub fn hash_bytes(data: &[u8]) -> Hash {
    blake3::hash(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_manifest() {
        let m = Manifest::new();
        assert!(m.is_empty());
        assert_eq!(m.len(), 0);
        assert_eq!(m.total_size(), 0);
    }

    #[test]
    fn record_and_check() {
        let mut m = Manifest::new();
        let hash = hash_bytes(b"hello android");
        m.record("/sdcard/file.txt", "/local/file.txt", hash, 13);

        assert_eq!(m.len(), 1);
        assert!(!m.is_empty());
        assert_eq!(m.total_size(), 13);

        // Same hash → no sync needed
        assert!(!m.needs_sync("/sdcard/file.txt", &hash.to_hex().to_string()));

        // Different hash → sync needed
        let new_hash = hash_bytes(b"modified content");
        assert!(m.needs_sync("/sdcard/file.txt", &new_hash.to_hex().to_string()));

        // Unknown key → sync needed
        assert!(m.needs_sync("/sdcard/other.txt", "anything"));
    }

    #[test]
    fn save_and_load() {
        let tmp = std::env::temp_dir().join(format!(
            "fudajiku_test_{}.json",
            std::process::id()
        ));

        let mut m = Manifest::new();
        let hash = hash_bytes(b"test data");
        m.record("key1", "/local/key1", hash, 9);
        m.save(&tmp).unwrap();

        let loaded = Manifest::load(&tmp);
        assert_eq!(loaded.len(), 1);
        assert!(!loaded.needs_sync("key1", &hash.to_hex().to_string()));

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn remove_entry() {
        let mut m = Manifest::new();
        let hash = hash_bytes(b"data");
        m.record("key", "/local", hash, 4);
        assert_eq!(m.len(), 1);

        m.remove("key");
        assert!(m.is_empty());
    }

    #[test]
    fn hash_consistency() {
        let h1 = hash_bytes(b"hello");
        let h2 = hash_bytes(b"hello");
        assert_eq!(h1, h2);

        let h3 = hash_bytes(b"world");
        assert_ne!(h1, h3);
    }
}
