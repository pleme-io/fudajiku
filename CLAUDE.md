# fudajiku — BLAKE3 Content-Addressed Manifest Tracking

> **★★★ CSE / Knowable Construction.** This repo operates under **Constructive Substrate Engineering** — canonical specification at [`pleme-io/theory/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md`](https://github.com/pleme-io/theory/blob/main/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md). The Compounding Directive (operational rules: solve once, load-bearing fixes only, idiom-first, models stay current, direction beats velocity) is in the org-level pleme-io/CLAUDE.md ★★★ section. Read both before non-trivial changes.


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
