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
//! * an existing file is only replaced when it is byte-identical to ours
//!   (a no-op reported as `unchanged`) or when `--force` says so — a
//!   local edit is not something an install command gets to overwrite.

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

use crate::output::{OutputFormat, print_output};

/// Every packaged skill, as (install-relative path, contents). The first
/// field is what is written under the target directory; the `include_str!`
/// path is the source file in the repo, whose directory name may differ
/// (the repo keeps services at `skills/<service>/`, the installed tree uses
/// the spec-mandated `grr-<service>/`).
const SKILLS: &[(&str, &str)] = &[
    ("grr/SKILL.md", include_str!("../../skills/grr/SKILL.md")),
    (
        "grr-analyticsadmin/SKILL.md",
        include_str!("../../skills/analyticsadmin/SKILL.md"),
    ),
    (
        "grr-analyticsdata/SKILL.md",
        include_str!("../../skills/analyticsdata/SKILL.md"),
    ),
    (
        "grr-calendar/SKILL.md",
        include_str!("../../skills/calendar/SKILL.md"),
    ),
    (
        "grr-chat/SKILL.md",
        include_str!("../../skills/chat/SKILL.md"),
    ),
    (
        "grr-docs/SKILL.md",
        include_str!("../../skills/docs/SKILL.md"),
    ),
    (
        "grr-drive/SKILL.md",
        include_str!("../../skills/drive/SKILL.md"),
    ),
    (
        "grr-forms/SKILL.md",
        include_str!("../../skills/forms/SKILL.md"),
    ),
    (
        "grr-gmail/SKILL.md",
        include_str!("../../skills/gmail/SKILL.md"),
    ),
    (
        "grr-people/SKILL.md",
        include_str!("../../skills/people/SKILL.md"),
    ),
    (
        "grr-script/SKILL.md",
        include_str!("../../skills/script/SKILL.md"),
    ),
    (
        "grr-searchconsole/SKILL.md",
        include_str!("../../skills/searchconsole/SKILL.md"),
    ),
    (
        "grr-sheets/SKILL.md",
        include_str!("../../skills/sheets/SKILL.md"),
    ),
    (
        "grr-slides/SKILL.md",
        include_str!("../../skills/slides/SKILL.md"),
    ),
    (
        "grr-tasks/SKILL.md",
        include_str!("../../skills/tasks/SKILL.md"),
    ),
    ("README.md", include_str!("../../skills/README.md")),
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
        if !report.skipped.is_empty() {
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
            let files: Vec<Value> = SKILLS
                .iter()
                .map(|&(rel, contents)| {
                    let state = match fs::read_to_string(target.join(rel)) {
                        Ok(existing) if existing == contents => "installed",
                        Ok(_) => "modified",
                        Err(_) => "missing",
                    };
                    json!({ "file": rel, "state": state })
                })
                .collect();
            json!({
                "target": target.display().to_string(),
                "exists": target.is_dir(),
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

#[derive(Debug, Default, PartialEq, Eq)]
pub struct InstallReport {
    pub target: PathBuf,
    /// Newly written (the path did not exist).
    pub installed: Vec<String>,
    /// Rewritten under `--force` (the path existed with different contents).
    pub updated: Vec<String>,
    /// Byte-identical to the embedded copy, left untouched.
    pub unchanged: Vec<String>,
    /// Left alone, with the reason.
    pub skipped: Vec<Skipped>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Skipped {
    pub path: String,
    pub reason: String,
}

/// Write the embedded skills under `target`. This is the testable core: it
/// takes the directory as an argument and never reads the home directory
/// itself, so tests can point it at a tempdir.
fn install_into(target: &Path, force: bool) -> Result<InstallReport> {
    let mut report = InstallReport {
        target: target.to_path_buf(),
        ..InstallReport::default()
    };

    for &(rel, contents) in SKILLS {
        // Enforce the spec's directory == frontmatter `name` rule here, so a
        // malformed embed is skipped rather than installed into the wrong (or
        // a colliding) directory. Every shipped file passes; see the tests.
        if let Err(reason) = check_skill_entry(rel, contents) {
            report.skipped.push(Skipped {
                path: rel.to_string(),
                reason,
            });
            continue;
        }

        let path = target.join(rel);
        match fs::read_to_string(&path) {
            Ok(existing) if existing == contents => report.unchanged.push(rel.to_string()),
            Ok(_) if !force => report.skipped.push(Skipped {
                path: rel.to_string(),
                reason: "exists with different contents; pass --force to replace".to_string(),
            }),
            Ok(_) => {
                fs::write(&path, contents)
                    .with_context(|| format!("writing {}", path.display()))?;
                report.updated.push(rel.to_string());
            }
            // Only a genuinely absent path is "new"; any other read error
            // (permissions, a directory in the way) is a skip, not a clobber.
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("creating {}", parent.display()))?;
                }
                fs::write(&path, contents)
                    .with_context(|| format!("writing {}", path.display()))?;
                report.installed.push(rel.to_string());
            }
            Err(err) => report.skipped.push(Skipped {
                path: rel.to_string(),
                reason: format!("cannot read existing file: {err}"),
            }),
        }
    }

    Ok(report)
}

fn print_install_report(report: &InstallReport) {
    println!("target: {}", report.target.display());
    for path in &report.installed {
        println!("  installed  {path}");
    }
    for path in &report.updated {
        println!("  updated    {path}");
    }
    for path in &report.unchanged {
        println!("  unchanged  {path}");
    }
    for skip in &report.skipped {
        println!("  skipped    {} ({})", skip.path, skip.reason);
    }
    println!(
        "{} files: {} installed, {} updated, {} unchanged, {} skipped",
        report.installed.len()
            + report.updated.len()
            + report.unchanged.len()
            + report.skipped.len(),
        report.installed.len(),
        report.updated.len(),
        report.unchanged.len(),
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
    fn modified_file_is_refused_then_replaced_with_force() {
        let dir = tempdir().unwrap();
        install_into(dir.path(), false).unwrap();

        let victim = dir.path().join("grr-gmail/SKILL.md");
        fs::write(&victim, "---\nname: grr-gmail\n---\nlocal edit\n").unwrap();

        // Without --force the edit survives and the report says skipped.
        let refused = install_into(dir.path(), false).unwrap();
        assert_eq!(
            refused.skipped,
            vec![Skipped {
                path: "grr-gmail/SKILL.md".to_string(),
                reason: "exists with different contents; pass --force to replace".to_string(),
            }]
        );
        assert_eq!(
            fs::read_to_string(&victim).unwrap(),
            "---\nname: grr-gmail\n---\nlocal edit\n"
        );

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
