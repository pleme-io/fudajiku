# fudajiku — BLAKE3 Content-Addressed Manifest Tracking

Tracks file sync state using BLAKE3 hashes. Determines which files
need updating by comparing content hashes. JSON persistence.

## API

```rust
let mut manifest = Manifest::load(Path::new(".manifest.json"));
let hash = hash_file(Path::new("file.txt"))?;

if manifest.needs_sync("remote/file.txt", &hash.to_hex().to_string()) {
    // sync the file...
    manifest.record("remote/file.txt", "local/file.txt", hash, size);
}

manifest.save(Path::new(".manifest.json"))?;
```

## Consumers

- `andro-sync` — file transfer manifest
- `nexus` — asset tracking
- `blackmatter-profiles` — image layer tracking
