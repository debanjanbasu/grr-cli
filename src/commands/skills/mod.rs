//! `grr skills` — install the packaged agent skills at the user level.
//!
//! The skill files are compiled into the binary (`include_str!`), so this
//! works offline and needs no config, no OAuth, and no network — the same
//! promise as `grr schema`. Installation is GLOBAL ONLY by design: the
//! cross-client user-level location is `~/.agents/skills/` (the
//! agentskills.io convention, read by Codex, opencode, Cursor, Gemini CLI,
//! Copilot CLI and omp). Claude Code is the one holdout — it reads only
//! `~/.claude/skills/` — so `--claude` mirrors there too. There is no
//! project-level install and no cwd detection: a skill that silently lands
//! in a repo is a skill nobody can find.
//!
//! Two invariants shape the writer:
//! * the spec requires a skill's directory name to equal its frontmatter
//!   `name`, so the install path is the frontmatter name (asserted for
//!   every embedded file, skipped if a future edit breaks it) — never the
//!   source directory's name;
//! * installs are self-migrating and provenance-aware (see [`migrate`]): a
//!   manifest beside the files records the packaged version and the bytes
//!   grr wrote, so a newer package can update untouched files, 3-way merge
//!   local edits, and never clobber a genuine conflict. `--force` still
//!   overwrites regardless.

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

use crate::output::{OutputFormat, print_output};

mod migrate;

use migrate::{INCOMING_SUFFIX, InstallReport, Manifest, SyncMode, classify};

/// Every packaged skill, as (install-relative path, contents). The first
/// field is what is written under the target directory; the `include_str!`
/// path is the source file in the repo, whose directory name may differ
/// (the repo keeps services at `skills/<service>/`, the installed tree uses
/// the spec-mandated `grr-<service>/`).
const SKILLS: &[(&str, &str)] = &[
    ("grr/SKILL.md", include_str!("../../../skills/grr/SKILL.md")),
    (
        "grr-analyticsadmin/SKILL.md",
        include_str!("../../../skills/analyticsadmin/SKILL.md"),
    ),
    (
        "grr-analyticsdata/SKILL.md",
        include_str!("../../../skills/analyticsdata/SKILL.md"),
    ),
    (
        "grr-calendar/SKILL.md",
        include_str!("../../../skills/calendar/SKILL.md"),
    ),
    (
        "grr-chat/SKILL.md",
        include_str!("../../../skills/chat/SKILL.md"),
    ),
    (
        "grr-docs/SKILL.md",
        include_str!("../../../skills/docs/SKILL.md"),
    ),
    (
        "grr-drive/SKILL.md",
        include_str!("../../../skills/drive/SKILL.md"),
    ),
    (
        "grr-forms/SKILL.md",
        include_str!("../../../skills/forms/SKILL.md"),
    ),
    (
        "grr-gmail/SKILL.md",
        include_str!("../../../skills/gmail/SKILL.md"),
    ),
    (
        "grr-people/SKILL.md",
        include_str!("../../../skills/people/SKILL.md"),
    ),
    (
        "grr-script/SKILL.md",
        include_str!("../../../skills/script/SKILL.md"),
    ),
    (
        "grr-searchconsole/SKILL.md",
        include_str!("../../../skills/searchconsole/SKILL.md"),
    ),
    (
        "grr-sheets/SKILL.md",
        include_str!("../../../skills/sheets/SKILL.md"),
    ),
    (
        "grr-slides/SKILL.md",
        include_str!("../../../skills/slides/SKILL.md"),
    ),
    (
        "grr-tasks/SKILL.md",
        include_str!("../../../skills/tasks/SKILL.md"),
    ),
    ("README.md", include_str!("../../../skills/README.md")),
];

#[derive(Subcommand, Debug)]
pub enum SkillsCommands {
    /// Install the packaged skills into the user-level skills directory
    /// (`~/.agents/skills/`), read by most agent harnesses
    Install(SkillsInstallArgs),
    /// Show the packaged skills and what each target directory holds
    List(SkillsListArgs),
}

#[derive(Args, Debug, Clone)]
pub struct SkillsInstallArgs {
    /// Install into this directory instead of ~/.agents/skills
    #[arg(long, value_name = "PATH")]
    pub dir: Option<PathBuf>,
    /// Also install into Claude Code's ~/.claude/skills (it reads no other dir)
    #[arg(long)]
    pub claude: bool,
    /// Replace files that exist with different contents
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Debug, Clone)]
pub struct SkillsListArgs {
    /// Inspect this directory instead of ~/.agents/skills
    #[arg(long, value_name = "PATH")]
    pub dir: Option<PathBuf>,
    /// Also inspect Claude Code's ~/.claude/skills
    #[arg(long)]
    pub claude: bool,
    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: OutputFormat,
}

pub fn handle_skills_cmd(cmd: SkillsCommands) -> Result<()> {
    match cmd {
        SkillsCommands::Install(args) => handle_install(args),
        SkillsCommands::List(args) => handle_list(args),
    }
}

fn handle_install(args: SkillsInstallArgs) -> Result<()> {
    let targets = resolve_targets(args.dir.as_deref(), args.claude)?;
    let mut refused: Vec<PathBuf> = Vec::new();
    for target in &targets {
        let report = install_into(target, args.force)?;
        print_install_report(&report);
        if !report.skipped.is_empty() || !report.conflicts.is_empty() {
            refused.push(report.target.clone());
        }
    }
    if !refused.is_empty() {
        // Non-zero exit on refusal: a scripted install must be able to tell
        // "everything is in place" from "I left your edits alone".
        bail!(
            "refused to overwrite existing files in {}; re-run with --force to replace them",
            refused
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(())
}

/// The opportunistic self-migration pass, called before the requested
/// command (see `cli::run`).
///
/// Laziness is the contract: [`migrate::migrate_dir`] reads the manifest,
/// compares the recorded version, and returns without hashing a single
/// file when they match. Only the two default target directories are
/// inspected — a custom `--dir` install is left alone, since grr cannot
/// know about directories it was never told about. Errors are swallowed:
/// a broken manifest must never break the command the user actually ran.
///
/// Returns one stderr-bound summary line when anything changed, else `None`.
pub(crate) fn maybe_auto_migrate() -> Option<String> {
    let home = dirs::home_dir()?;
    let version = env!("CARGO_PKG_VERSION");
    let mut lines: Vec<String> = Vec::new();
    for dir in [agents_skills_dir(&home), claude_skills_dir(&home)] {
        if let Some(line) = migrate::migrate_dir(&dir, SKILLS, version) {
            lines.push(line);
        }
    }
    (!lines.is_empty()).then(|| lines.join("; "))
}

fn handle_list(args: SkillsListArgs) -> Result<()> {
    let targets = resolve_targets(args.dir.as_deref(), args.claude)?;

    let skills: Vec<Value> = SKILLS
        .iter()
        .map(|&(rel, contents)| {
            json!({
                "name": frontmatter_name(contents),
                "file": rel,
            })
        })
        .collect();

    let installations: Vec<Value> = targets
        .iter()
        .map(|target| {
            let manifest = Manifest::load(target);
            let files: Vec<Value> = SKILLS
                .iter()
                .map(|&(rel, contents)| {
                    let state = classify(target, rel, contents, manifest.as_ref());
                    json!({ "file": rel, "state": state })
                })
                .collect();
            json!({
                "target": target.display().to_string(),
                "exists": target.is_dir(),
                "installed_version": manifest.as_ref().map(|m| m.version.clone()),
                "files": files,
            })
        })
        .collect();

    print_output(
        &json!({
            "skill_count": SKILLS.len(),
            "skills": skills,
            "installations": installations,
        }),
        args.format,
    )?;
    Ok(())
}

/// The cross-client user-level skills directory (agentskills.io spec).
fn agents_skills_dir(home: &Path) -> PathBuf {
    home.join(".agents").join("skills")
}

/// Claude Code reads its own directory and nothing else.
fn claude_skills_dir(home: &Path) -> PathBuf {
    home.join(".claude").join("skills")
}

fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().context("could not determine the home directory; pass --dir")
}

/// Resolve the directories to write to. `--dir` replaces the default
/// `~/.agents/skills`; `--claude` adds (or dedupes) `~/.claude/skills`.
/// `home` is injected so tests never read the real home directory.
fn resolve_targets_from(home: &Path, dir: Option<&Path>, claude: bool) -> Vec<PathBuf> {
    let primary = match dir {
        Some(dir) => dir.to_path_buf(),
        None => agents_skills_dir(home),
    };
    let mut targets = vec![primary];
    if claude {
        let claude = claude_skills_dir(home);
        if !targets.contains(&claude) {
            targets.push(claude);
        }
    }
    targets
}

fn resolve_targets(dir: Option<&Path>, claude: bool) -> Result<Vec<PathBuf>> {
    // An explicit directory with no Claude mirror needs no home lookup at
    // all — the smoke test path must work on a machine with no home.
    match (dir, claude) {
        (Some(dir), false) => Ok(vec![dir.to_path_buf()]),
        _ => Ok(resolve_targets_from(&home_dir()?, dir, claude)),
    }
}

/// Run the embedded-skill install/upgrade pass over `target`. The
/// provenance-aware engine lives in [`migrate`]; this is the thin binding
/// that names the current package version and `SKILLS`.
fn install_into(target: &Path, force: bool) -> Result<InstallReport> {
    migrate::sync_into(
        target,
        SKILLS,
        env!("CARGO_PKG_VERSION"),
        force,
        SyncMode::Explicit,
    )
}

fn print_install_report(report: &InstallReport) {
    println!("target: {}", report.target.display());
    for path in &report.installed {
        println!("  installed  {path}");
    }
    for path in &report.updated {
        println!("  updated    {path}");
    }
    for path in &report.merged {
        println!("  merged     {path}");
    }
    for path in &report.unchanged {
        println!("  unchanged  {path}");
    }
    for path in &report.conflicts {
        println!("  conflict   {path} (packaged-new kept as {path}{INCOMING_SUFFIX})");
    }
    for skip in &report.skipped {
        println!("  skipped    {} ({})", skip.path, skip.reason);
    }
    println!(
        "{} files: {} installed, {} updated, {} merged, {} unchanged, {} conflicts, {} skipped",
        report.installed.len()
            + report.updated.len()
            + report.merged.len()
            + report.unchanged.len()
            + report.conflicts.len()
            + report.skipped.len(),
        report.installed.len(),
        report.updated.len(),
        report.merged.len(),
        report.unchanged.len(),
        report.conflicts.len(),
        report.skipped.len(),
    );
}

/// The `name:` field of a skill's YAML frontmatter, if there is one.
fn frontmatter_name(contents: &str) -> Option<&str> {
    let mut lines = contents.lines();
    if lines.next()?.trim_end() != "---" {
        return None;
    }
    for line in lines {
        let line = line.trim();
        if line == "---" {
            return None;
        }
        if let Some(rest) = line.strip_prefix("name:") {
            let name = rest.trim().trim_matches(['"', '\'']);
            return (!name.is_empty()).then_some(name);
        }
    }
    None
}

/// agentskills.io names: lowercase alphanumerics separated by single hyphens.
fn is_valid_skill_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.split('-').all(|segment| {
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

/// SKILL.md files must declare a valid `name` equal to their directory.
/// README.md is the plain index and carries no frontmatter.
fn check_skill_entry(rel: &str, contents: &str) -> Result<(), String> {
    if !rel.ends_with("SKILL.md") {
        return Ok(());
    }
    let dir = rel.split_once('/').map(|(dir, _)| dir).unwrap_or_default();
    let Some(name) = frontmatter_name(contents) else {
        return Err("no `name:` in frontmatter".to_string());
    };
    if !is_valid_skill_name(name) {
        return Err(format!("invalid skill name `{name}`"));
    }
    if name != dir {
        return Err(format!("frontmatter name `{name}` != directory `{dir}`"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn embedded(rel: &str) -> &'static str {
        SKILLS
            .iter()
            .find(|&&(r, _)| r == rel)
            .map(|&(_, contents)| contents)
            .expect("embedded skill")
    }

    #[test]
    fn embedded_skills_are_spec_conformant() {
        assert_eq!(SKILLS.len(), 16);
        for &(rel, contents) in SKILLS {
            check_skill_entry(rel, contents).unwrap_or_else(|err| panic!("{rel}: {err}"));
        }
        // And every SKILL.md's directory really is its frontmatter name.
        for &(rel, contents) in SKILLS {
            if rel.ends_with("SKILL.md") {
                let dir = rel.split_once('/').unwrap().0;
                assert_eq!(frontmatter_name(contents), Some(dir));
            }
        }
    }

    #[test]
    fn install_writes_every_file_then_is_idempotent() {
        let dir = tempdir().unwrap();
        let first = install_into(dir.path(), false).unwrap();
        assert_eq!(first.installed.len(), SKILLS.len());
        assert!(first.updated.is_empty());
        assert!(first.unchanged.is_empty());
        assert!(first.skipped.is_empty());

        for &(rel, contents) in SKILLS {
            assert_eq!(fs::read_to_string(dir.path().join(rel)).unwrap(), contents);
        }

        let second = install_into(dir.path(), false).unwrap();
        assert!(second.installed.is_empty());
        assert!(second.updated.is_empty());
        assert_eq!(second.unchanged.len(), SKILLS.len());
        assert!(second.skipped.is_empty());
    }

    #[test]
    fn a_local_edit_survives_a_same_version_install_and_force_replaces_it() {
        let dir = tempdir().unwrap();
        install_into(dir.path(), false).unwrap();

        let victim = dir.path().join("grr-gmail/SKILL.md");
        let edited = "---\nname: grr-gmail\n---\nlocal edit\n";
        fs::write(&victim, edited).unwrap();

        // The file is provenance-tracked, so a same-version pass 3-way merges
        // (base == packaged == theirs): the local edit is preserved, nothing
        // is written, and the report does not refuse.
        let merged = install_into(dir.path(), false).unwrap();
        assert!(merged.unchanged.contains(&"grr-gmail/SKILL.md".to_string()));
        assert!(merged.skipped.is_empty());
        assert!(merged.conflicts.is_empty());
        assert!(merged.updated.is_empty());
        assert_eq!(fs::read_to_string(&victim).unwrap(), edited);

        // With --force the embedded copy comes back, reported as updated.
        let forced = install_into(dir.path(), true).unwrap();
        assert_eq!(forced.updated, vec!["grr-gmail/SKILL.md".to_string()]);
        assert!(forced.skipped.is_empty());
        assert_eq!(
            fs::read_to_string(&victim).unwrap(),
            embedded("grr-gmail/SKILL.md")
        );
    }

    #[test]
    fn a_file_without_provenance_is_refused_then_replaced_with_force() {
        // A pre-manifest install (or a hand-placed file) is not grr's to
        // touch: no manifest entry means no merge, and no silent overwrite.
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("grr-gmail")).unwrap();
        let victim = dir.path().join("grr-gmail/SKILL.md");
        fs::write(&victim, "---\nname: grr-gmail\n---\nhand written\n").unwrap();

        let refused = install_into(dir.path(), false).unwrap();
        assert!(
            refused
                .skipped
                .iter()
                .any(|skip| skip.path == "grr-gmail/SKILL.md")
        );
        assert_eq!(
            fs::read_to_string(&victim).unwrap(),
            "---\nname: grr-gmail\n---\nhand written\n"
        );

        let forced = install_into(dir.path(), true).unwrap();
        assert!(forced.updated.contains(&"grr-gmail/SKILL.md".to_string()));
        assert_eq!(
            fs::read_to_string(&victim).unwrap(),
            embedded("grr-gmail/SKILL.md")
        );
    }

    #[test]
    fn install_records_provenance_for_every_packaged_file() {
        let dir = tempdir().unwrap();
        install_into(dir.path(), false).unwrap();

        let manifest = Manifest::load(dir.path()).expect("manifest written on install");
        assert_eq!(manifest.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(manifest.files.len(), SKILLS.len());
        for &(rel, contents) in SKILLS {
            let entry = &manifest.files[rel];
            let sha = migrate::sha256_hex(contents.as_bytes());
            assert_eq!(entry.base_sha256, sha, "base sha for {rel}");
            assert_eq!(entry.written_sha256, sha, "written sha for {rel}");
            // The reconstructable merge base is on disk beside the files.
            assert_eq!(
                fs::read_to_string(dir.path().join(".grr-backup").join(rel)).unwrap(),
                contents,
                "backup for {rel}"
            );
        }
    }

    #[test]
    fn targets_are_home_relative_and_deduped() {
        let home = Path::new("/home/tester");
        assert_eq!(
            agents_skills_dir(home),
            PathBuf::from("/home/tester/.agents/skills")
        );
        assert_eq!(
            claude_skills_dir(home),
            PathBuf::from("/home/tester/.claude/skills")
        );

        // Default target plus the Claude mirror.
        assert_eq!(
            resolve_targets_from(home, None, true),
            vec![
                PathBuf::from("/home/tester/.agents/skills"),
                PathBuf::from("/home/tester/.claude/skills"),
            ]
        );

        // An explicit --dir that already is the Claude directory stays a
        // single target, so `--dir ~/.claude/skills --claude` writes once.
        let claude = claude_skills_dir(home);
        assert_eq!(
            resolve_targets_from(home, Some(&claude), true),
            vec![claude]
        );
    }

    #[test]
    fn explicit_dir_replaces_the_default() {
        let home = Path::new("/home/tester");
        assert_eq!(
            resolve_targets_from(home, Some(Path::new("/tmp/skills-smoke")), false),
            vec![PathBuf::from("/tmp/skills-smoke")]
        );
        assert_eq!(
            resolve_targets_from(home, Some(Path::new("/tmp/skills-smoke")), true),
            vec![
                PathBuf::from("/tmp/skills-smoke"),
                PathBuf::from("/home/tester/.claude/skills"),
            ]
        );
    }

    #[test]
    fn check_skill_entry_rejects_bad_names() {
        // Directory != frontmatter name.
        assert!(check_skill_entry("gmail/SKILL.md", "---\nname: grr-gmail\n---\n").is_err());
        // Missing frontmatter, or an invalid name shape.
        assert!(check_skill_entry("grr/SKILL.md", "# no frontmatter").is_err());
        assert!(check_skill_entry("grr/SKILL.md", "---\nname: Grr!\n---\n").is_err());
        // The README is exempt.
        assert!(check_skill_entry("README.md", "# index").is_ok());
        assert!(check_skill_entry("grr/SKILL.md", "---\nname: grr\n---\n").is_ok());
    }

    #[test]
    fn frontmatter_name_reads_quoted_and_bare_values() {
        assert_eq!(frontmatter_name("---\nname: grr\n---\nbody"), Some("grr"));
        assert_eq!(
            frontmatter_name("---\ndescription: x\nname: \"grr-gmail\"\n---\n"),
            Some("grr-gmail")
        );
        assert_eq!(frontmatter_name("# just a readme"), None);
        assert_eq!(frontmatter_name("---\ndescription: x\n---\n"), None);
    }

    #[test]
    fn unreadable_path_is_skipped_not_clobbered() {
        // A directory sitting where a skill file belongs can never be read
        // as a file; it must be reported, not deleted or overwritten.
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("grr/SKILL.md")).unwrap();
        let report = install_into(dir.path(), true).unwrap();
        assert!(
            report
                .skipped
                .iter()
                .any(|skip| skip.path == "grr/SKILL.md")
        );
        assert!(dir.path().join("grr/SKILL.md").is_dir());
    }
}
