//! Skill install provenance and self-migration.
//!
//! `grr skills install` records what it wrote in a manifest at
//! `<target>/.grr-skills.json`: the package version plus, per packaged
//! file, the sha256 of the packaged content it installed (the 3-way merge
//! base) and the sha256 of the exact bytes it wrote (what "untouched by
//! the user" means). A copy of each packaged file is kept under
//! `<target>/.grr-backup/<relpath>` so the base can be reconstructed.
//!
//! Base-content strategy: we store the base bytes, not "re-derive them
//! from the old packaged set". A binary only carries the CURRENT packaged
//! files (`include_str!`), so re-deriving would be impossible after
//! jumping more than one version — the backup is the only correct base
//! when the recorded version is several releases behind. The backup is
//! refreshed on every write, so it always holds the packaged content of
//! the recorded version.
//!
//! Two entry points:
//! * [`sync_into`] — the install/upgrade pass (explicit `grr skills install`
//!   or the lazy auto-migration).
//! * [`migrate_dir`] — the opportunistic pass. It is LAZY: one stat for
//!   the manifest, a tiny JSON compare, and only if the recorded version
//!   differs does anything hash a file.
//!
//! The manifest and the backup are per target directory. The `--claude`
//! mirror (`~/.claude/skills`) therefore keeps its own, so the two trees
//! never share provenance and either one can be migrated independently.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Provenance manifest, written beside the installed files.
pub(crate) const MANIFEST_NAME: &str = ".grr-skills.json";

/// Packaged-content snapshots (the 3-way merge base) live here.
pub(crate) const BACKUP_DIR: &str = ".grr-backup";

/// Suffix of the packaged-new copy left beside a conflicted file.
pub(crate) const INCOMING_SUFFIX: &str = ".grr-incoming";

#[cfg(test)]
thread_local! {
    /// Counts `sha256_hex` calls so a test can prove the version fast path
    /// does no hashing at all.
    static HASH_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn reset_hash_calls() {
    HASH_CALLS.with(|calls| calls.set(0));
}

#[cfg(test)]
pub(crate) fn hash_calls() -> u64 {
    HASH_CALLS.with(std::cell::Cell::get)
}

/// Lowercase hex sha256. Pure Rust; no extra digest-to-hex dependency.
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    #[cfg(test)]
    HASH_CALLS.with(|calls| calls.set(calls.get() + 1));
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest.as_slice() {
        use std::fmt::Write as _;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// One file's recorded provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileEntry {
    /// sha256 of the packaged content at the recorded `version` (the merge base).
    pub base_sha256: String,
    /// sha256 of the exact bytes grr last wrote for this path.
    pub written_sha256: String,
}

/// The set of files grr owns under one target directory.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Manifest {
    pub version: String,
    #[serde(default)]
    pub files: BTreeMap<String, FileEntry>,
}

impl Manifest {
    pub(crate) fn path(target: &Path) -> PathBuf {
        target.join(MANIFEST_NAME)
    }

    /// Read the manifest, or `None` when absent or unparseable. A corrupt
    /// manifest is treated as "not installed" — never as a reason to fail.
    pub(crate) fn load(target: &Path) -> Option<Manifest> {
        let text = fs::read_to_string(Self::path(target)).ok()?;
        serde_json::from_str(&text).ok()
    }

    pub(crate) fn save(&self, target: &Path) -> Result<()> {
        let path = Self::path(target);
        let body = serde_json::to_string_pretty(self).context("serializing the skills manifest")?;
        write_file(&path, &body)
    }
}

/// Whether [`sync_into`] is the explicit install command or the lazy
/// version-change pass. The only behavioral difference: the lazy pass
/// never resurrects a file the user deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SyncMode {
    Explicit,
    Migrate,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct InstallReport {
    pub target: PathBuf,
    /// Newly written (the path did not exist).
    pub installed: Vec<String>,
    /// Replaced with the new packaged content (untouched local, safe path).
    pub updated: Vec<String>,
    /// A 3-way merge combined local edits with the new packaged content.
    pub merged: Vec<String>,
    /// Byte-identical to the packaged copy, left untouched.
    pub unchanged: Vec<String>,
    /// Left alone, with the reason.
    pub skipped: Vec<Skipped>,
    /// Left untouched; the packaged-new content was written beside them.
    pub conflicts: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Skipped {
    pub path: String,
    pub reason: String,
}

impl InstallReport {
    fn wrote_anything(&self) -> bool {
        !self.installed.is_empty()
            || !self.updated.is_empty()
            || !self.merged.is_empty()
            || !self.conflicts.is_empty()
    }
}

/// The install/upgrade pass over one target directory.
///
/// Per file, given the packaged-new content `P`, the recorded base `B` (a
/// backup snapshot) and the local file `L`:
/// * `L == P` → current, no write;
/// * `--force` → overwrite with `P`, no merge (the flag's original meaning);
/// * `L` is exactly what grr last wrote and that write was pure `P_old`
///   (`written == base`) → safe auto-update to `P`;
/// * otherwise → 3-way merge `diffy::merge(B, L, P)`: clean merges write
///   the result, conflicts leave `L` untouched and drop `P` beside it as
///   `<name>.grr-incoming`.
///
/// A file with no manifest entry is not ours: it is skipped, never merged
/// or overwritten (unless `--force` says override everything).
pub(crate) fn sync_into(
    target: &Path,
    packaged: &[(&str, &str)],
    version: &str,
    force: bool,
    mode: SyncMode,
) -> Result<InstallReport> {
    let mut report = InstallReport {
        target: target.to_path_buf(),
        ..InstallReport::default()
    };
    let loaded = Manifest::load(target);
    let mut manifest = loaded.clone().unwrap_or_default();
    let mut dirty = false;

    for &(rel, contents) in packaged {
        // Same spec guard as the writer: a malformed embed is skipped, not
        // installed into the wrong directory.
        if let Err(reason) = super::check_skill_entry(rel, contents) {
            report.skipped.push(Skipped {
                path: rel.to_string(),
                reason,
            });
            continue;
        }

        let path = target.join(rel);
        let packaged_sha = sha256_hex(contents.as_bytes());
        let entry = manifest.files.get(rel).cloned();

        let local = match fs::read_to_string(&path) {
            Ok(local) => Some(local),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
            Err(err) => {
                report.skipped.push(Skipped {
                    path: rel.to_string(),
                    reason: format!("cannot read existing file: {err}"),
                });
                continue;
            }
        };

        let Some(local) = local else {
            // Absent. grr owns it (entry present) but the user deleted it:
            // the lazy pass respects that; an explicit install restores it.
            if mode == SyncMode::Migrate && entry.is_some() {
                report.skipped.push(Skipped {
                    path: rel.to_string(),
                    reason: "removed locally; rerun `grr skills install` to restore".to_string(),
                });
                continue;
            }
            write_file(&path, contents)?;
            report.installed.push(rel.to_string());
            manifest.files.insert(
                rel.to_string(),
                FileEntry {
                    base_sha256: packaged_sha.clone(),
                    written_sha256: packaged_sha.clone(),
                },
            );
            write_backup(target, rel, contents)?;
            dirty = true;
            continue;
        };

        if local == contents {
            report.unchanged.push(rel.to_string());
            if entry
                .as_ref()
                .is_none_or(|e| e.base_sha256 != packaged_sha || e.written_sha256 != packaged_sha)
            {
                manifest.files.insert(
                    rel.to_string(),
                    FileEntry {
                        base_sha256: packaged_sha.clone(),
                        written_sha256: packaged_sha.clone(),
                    },
                );
                write_backup(target, rel, contents)?;
                dirty = true;
            }
            continue;
        }

        if force {
            write_file(&path, contents)?;
            report.updated.push(rel.to_string());
            manifest.files.insert(
                rel.to_string(),
                FileEntry {
                    base_sha256: packaged_sha.clone(),
                    written_sha256: packaged_sha.clone(),
                },
            );
            write_backup(target, rel, contents)?;
            dirty = true;
            continue;
        }

        let Some(entry) = entry else {
            // Different content, no provenance: not ours. Never touch it.
            report.skipped.push(Skipped {
                path: rel.to_string(),
                reason:
                    "exists with different contents; not installed by grr — pass --force to replace"
                        .to_string(),
            });
            continue;
        };

        let local_sha = sha256_hex(local.as_bytes());

        // Safe auto-update: the local bytes are exactly what grr last wrote,
        // and that write was pure packaged content (neither a merge result
        // nor a user edit).
        if local_sha == entry.written_sha256 && entry.written_sha256 == entry.base_sha256 {
            write_file(&path, contents)?;
            report.updated.push(rel.to_string());
            manifest.files.insert(
                rel.to_string(),
                FileEntry {
                    base_sha256: packaged_sha.clone(),
                    written_sha256: packaged_sha.clone(),
                },
            );
            write_backup(target, rel, contents)?;
            dirty = true;
            continue;
        }

        let Some(base) = read_backup(target, rel) else {
            report.skipped.push(Skipped {
                path: rel.to_string(),
                reason: "no recorded base to merge against; pass --force to replace".to_string(),
            });
            continue;
        };

        match diffy::merge(&base, &local, contents) {
            Ok(merged) if merged == local => {
                // The packaged side did not change this file's merged
                // region; nothing to write, but advance the base.
                report.unchanged.push(rel.to_string());
                manifest.files.insert(
                    rel.to_string(),
                    FileEntry {
                        base_sha256: packaged_sha.clone(),
                        written_sha256: entry.written_sha256.clone(),
                    },
                );
                write_backup(target, rel, contents)?;
                dirty = true;
            }
            Ok(merged) => {
                write_file(&path, &merged)?;
                report.merged.push(rel.to_string());
                manifest.files.insert(
                    rel.to_string(),
                    FileEntry {
                        base_sha256: packaged_sha.clone(),
                        written_sha256: sha256_hex(merged.as_bytes()),
                    },
                );
                write_backup(target, rel, contents)?;
                dirty = true;
            }
            Err(_) => {
                // Conflict: the user's file is inviolable. Drop the new
                // packaged copy beside it and advance the base so the next
                // version merges from the right point.
                let incoming = incoming_path(&path);
                write_file(&incoming, contents)?;
                report.conflicts.push(rel.to_string());
                manifest.files.insert(
                    rel.to_string(),
                    FileEntry {
                        base_sha256: packaged_sha.clone(),
                        written_sha256: entry.written_sha256.clone(),
                    },
                );
                write_backup(target, rel, contents)?;
                dirty = true;
            }
        }
    }

    if dirty || manifest.version != version {
        manifest.version = version.to_string();
        manifest.save(target)?;
    }

    Ok(report)
}

/// The lazy version-change pass over one target directory.
///
/// Fast path, in order: read the manifest (a stat plus a tiny JSON read);
/// bail if absent/unparseable; bail if the recorded version already equals
/// the binary's. Only then does a full [`sync_into`] run — and only that
/// pass hashes anything.
pub(crate) fn migrate_dir(dir: &Path, packaged: &[(&str, &str)], version: &str) -> Option<String> {
    let text = fs::read_to_string(Manifest::path(dir)).ok()?;
    let manifest: Manifest = serde_json::from_str(&text).ok()?;
    if manifest.version == version {
        return None;
    }
    let from = manifest.version.clone();
    let report = sync_into(dir, packaged, version, false, SyncMode::Migrate).ok()?;
    summarize(&report, &from, version)
}

fn summarize(report: &InstallReport, from: &str, to: &str) -> Option<String> {
    if !report.wrote_anything() {
        return None;
    }
    let mut parts = Vec::new();
    if !report.installed.is_empty() {
        parts.push(format!("{} installed", report.installed.len()));
    }
    if !report.updated.is_empty() {
        parts.push(format!("{} updated", report.updated.len()));
    }
    if !report.merged.is_empty() {
        parts.push(format!("{} merged", report.merged.len()));
    }
    if !report.conflicts.is_empty() {
        parts.push(format!(
            "{} conflicts (new copy kept as {INCOMING_SUFFIX})",
            report.conflicts.len()
        ));
    }
    let from = if from.is_empty() { "unknown" } else { from };
    Some(format!(
        "grr: skills migrated {from} -> {to} ({}); local edits were preserved",
        parts.join(", ")
    ))
}

/// The per-file state `grr skills list` reports for one target.
pub(crate) fn classify(
    target: &Path,
    rel: &str,
    packaged: &str,
    manifest: Option<&Manifest>,
) -> &'static str {
    match fs::read_to_string(target.join(rel)) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => "missing",
        Err(_) => "unreadable",
        Ok(local) => {
            if local == packaged {
                return "current";
            }
            match manifest.and_then(|m| m.files.get(rel)) {
                Some(entry) if sha256_hex(local.as_bytes()) == entry.written_sha256 => "outdated",
                Some(_) => "locally-modified",
                None => "unmanaged",
            }
        }
    }
}

fn backup_path(target: &Path, rel: &str) -> PathBuf {
    target.join(BACKUP_DIR).join(rel)
}

fn read_backup(target: &Path, rel: &str) -> Option<String> {
    fs::read_to_string(backup_path(target, rel)).ok()
}

fn write_backup(target: &Path, rel: &str, contents: &str) -> Result<()> {
    write_file(&backup_path(target, rel), contents)
}

fn incoming_path(path: &Path) -> PathBuf {
    let mut raw = path.as_os_str().to_owned();
    raw.push(INCOMING_SUFFIX);
    PathBuf::from(raw)
}

fn write_file(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::write(path, contents).with_context(|| format!("writing {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// A packaged-file table with no frontmatter constraints (only
    /// `SKILL.md` entries are validated), so tests can use plain bodies.
    fn packaged(body: &'static str) -> [(&'static str, &'static str); 1] {
        [("grr/body.md", body)]
    }

    fn read(target: &Path, rel: &str) -> String {
        fs::read_to_string(target.join(rel)).unwrap()
    }

    #[test]
    fn unchanged_install_is_idempotent_and_records_the_manifest() {
        let dir = tempdir().unwrap();
        let first = sync_into(
            dir.path(),
            &packaged("hello\n"),
            "1.0.0",
            false,
            SyncMode::Explicit,
        )
        .unwrap();
        assert_eq!(first.installed, vec!["grr/body.md".to_string()]);
        assert!(first.updated.is_empty() && first.merged.is_empty() && first.conflicts.is_empty());

        let manifest = Manifest::load(dir.path()).unwrap();
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(
            manifest.files["grr/body.md"].base_sha256,
            sha256_hex(b"hello\n")
        );
        assert_eq!(
            manifest.files["grr/body.md"].written_sha256,
            sha256_hex(b"hello\n")
        );
        // The merge base snapshot is on disk.
        assert_eq!(read(dir.path(), ".grr-backup/grr/body.md"), "hello\n");

        let second = sync_into(
            dir.path(),
            &packaged("hello\n"),
            "1.0.0",
            false,
            SyncMode::Explicit,
        )
        .unwrap();
        assert_eq!(second.unchanged, vec!["grr/body.md".to_string()]);
        assert!(second.installed.is_empty() && second.updated.is_empty());
    }

    #[test]
    fn new_version_safe_auto_updates_an_untouched_file() {
        let dir = tempdir().unwrap();
        sync_into(
            dir.path(),
            &packaged("v1\n"),
            "1.0.0",
            false,
            SyncMode::Explicit,
        )
        .unwrap();

        // No local edit; the packaged file changed.
        let report = sync_into(
            dir.path(),
            &packaged("v2\n"),
            "2.0.0",
            false,
            SyncMode::Explicit,
        )
        .unwrap();
        assert_eq!(report.updated, vec!["grr/body.md".to_string()]);
        assert_eq!(read(dir.path(), "grr/body.md"), "v2\n");
        assert_eq!(Manifest::load(dir.path()).unwrap().version, "2.0.0");
    }

    #[test]
    fn clean_merge_combines_local_edits_with_the_new_packaged_content() {
        let dir = tempdir().unwrap();
        sync_into(
            dir.path(),
            &packaged("alpha\nbravo\ncharlie\n"),
            "1.0.0",
            false,
            SyncMode::Explicit,
        )
        .unwrap();

        // Local edits the first line; the new package edits the last.
        fs::write(dir.path().join("grr/body.md"), "ALPHA\nbravo\ncharlie\n").unwrap();
        let report = sync_into(
            dir.path(),
            &packaged("alpha\nbravo\nCHARLIE\n"),
            "2.0.0",
            false,
            SyncMode::Explicit,
        )
        .unwrap();

        assert_eq!(report.merged, vec!["grr/body.md".to_string()]);
        let merged = read(dir.path(), "grr/body.md");
        assert!(merged.contains("ALPHA"), "local edit lost: {merged}");
        assert!(merged.contains("CHARLIE"), "packaged edit lost: {merged}");
        assert!(report.conflicts.is_empty());
        // written hash reflects the merged bytes, not the packaged ones.
        let manifest = Manifest::load(dir.path()).unwrap();
        assert_eq!(
            manifest.files["grr/body.md"].written_sha256,
            sha256_hex(merged.as_bytes())
        );
        assert_eq!(
            manifest.files["grr/body.md"].base_sha256,
            sha256_hex(b"alpha\nbravo\nCHARLIE\n")
        );
    }

    #[test]
    fn conflicting_edit_leaves_the_local_file_and_drops_an_incoming_copy() {
        let dir = tempdir().unwrap();
        sync_into(
            dir.path(),
            &packaged("alpha\nbravo\ncharlie\n"),
            "1.0.0",
            false,
            SyncMode::Explicit,
        )
        .unwrap();

        // Both sides change the SAME line differently.
        fs::write(dir.path().join("grr/body.md"), "LOCAL\nbravo\ncharlie\n").unwrap();
        let report = sync_into(
            dir.path(),
            &packaged("PACKAGED\nbravo\ncharlie\n"),
            "2.0.0",
            false,
            SyncMode::Explicit,
        )
        .unwrap();

        assert_eq!(report.conflicts, vec!["grr/body.md".to_string()]);
        assert!(report.merged.is_empty() && report.updated.is_empty());
        // The user's file is untouched...
        assert_eq!(read(dir.path(), "grr/body.md"), "LOCAL\nbravo\ncharlie\n");
        // ...and the packaged-new copy sits beside it.
        assert_eq!(
            read(dir.path(), "grr/body.md.grr-incoming"),
            "PACKAGED\nbravo\ncharlie\n"
        );
    }

    #[test]
    fn force_still_overwrites_a_local_edit() {
        let dir = tempdir().unwrap();
        sync_into(
            dir.path(),
            &packaged("v1\n"),
            "1.0.0",
            false,
            SyncMode::Explicit,
        )
        .unwrap();
        fs::write(dir.path().join("grr/body.md"), "local\n").unwrap();

        let report = sync_into(
            dir.path(),
            &packaged("v2\n"),
            "2.0.0",
            true,
            SyncMode::Explicit,
        )
        .unwrap();
        assert_eq!(report.updated, vec!["grr/body.md".to_string()]);
        assert_eq!(read(dir.path(), "grr/body.md"), "v2\n");
    }

    #[test]
    fn files_without_provenance_are_never_touched() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("grr")).unwrap();
        fs::write(dir.path().join("grr/body.md"), "not grr's file\n").unwrap();

        let report = sync_into(
            dir.path(),
            &packaged("v1\n"),
            "1.0.0",
            false,
            SyncMode::Explicit,
        )
        .unwrap();
        assert_eq!(report.skipped.len(), 1);
        assert_eq!(read(dir.path(), "grr/body.md"), "not grr's file\n");
    }

    #[test]
    fn auto_migration_runs_only_when_the_recorded_version_differs() {
        let dir = tempdir().unwrap();
        sync_into(
            dir.path(),
            &packaged("v1\n"),
            "1.0.0",
            false,
            SyncMode::Explicit,
        )
        .unwrap();

        // Same version: the fast path short-circuits with NO hashing.
        reset_hash_calls();
        assert_eq!(migrate_dir(dir.path(), &packaged("v2\n"), "1.0.0"), None);
        assert_eq!(hash_calls(), 0, "same-version fast path must not hash");
        assert_eq!(read(dir.path(), "grr/body.md"), "v1\n");

        // Different version: the pass runs, updates the untouched file and
        // reports one summary line.
        let summary = migrate_dir(dir.path(), &packaged("v2\n"), "2.0.0").unwrap();
        assert!(summary.contains("1.0.0 -> 2.0.0"), "{summary}");
        assert!(summary.contains("1 updated"), "{summary}");
        assert_eq!(read(dir.path(), "grr/body.md"), "v2\n");
    }

    #[test]
    fn auto_migration_never_resurrects_a_locally_deleted_file() {
        let dir = tempdir().unwrap();
        sync_into(
            dir.path(),
            &packaged("v1\n"),
            "1.0.0",
            false,
            SyncMode::Explicit,
        )
        .unwrap();
        fs::remove_file(dir.path().join("grr/body.md")).unwrap();

        assert_eq!(migrate_dir(dir.path(), &packaged("v2\n"), "2.0.0"), None);
        assert!(!dir.path().join("grr/body.md").exists());
    }

    #[test]
    fn missing_manifest_means_nothing_to_migrate() {
        let dir = tempdir().unwrap();
        assert_eq!(migrate_dir(dir.path(), &packaged("v1\n"), "2.0.0"), None);
    }

    #[test]
    fn classify_separates_outdated_from_locally_modified() {
        let dir = tempdir().unwrap();
        sync_into(
            dir.path(),
            &packaged("v1\n"),
            "1.0.0",
            false,
            SyncMode::Explicit,
        )
        .unwrap();
        let manifest = Manifest::load(dir.path());

        // Untouched -> outdated when the packaged content moved.
        assert_eq!(
            classify(dir.path(), "grr/body.md", "v2\n", manifest.as_ref()),
            "outdated"
        );
        // Local edit -> locally-modified.
        fs::write(dir.path().join("grr/body.md"), "mine\n").unwrap();
        assert_eq!(
            classify(dir.path(), "grr/body.md", "v2\n", manifest.as_ref()),
            "locally-modified"
        );
        // Same as packaged -> current; absent -> missing.
        assert_eq!(
            classify(dir.path(), "grr/body.md", "mine\n", manifest.as_ref()),
            "current"
        );
        assert_eq!(
            classify(dir.path(), "grr/nope.md", "v2\n", manifest.as_ref()),
            "missing"
        );
    }
}
