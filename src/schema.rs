//! `grr schema`: dump the command tree as JSON.
//!
//! The binary is its own contract (inspired by gogcli's "discover the
//! contract"): agents and docs generators read this instead of scraping
//! --help text.

use anyhow::Result;
use clap::{Arg, Command};
use serde_json::{Value, json};

use crate::output::{OutputFormat, print_output};

#[derive(clap::Args, Debug, Clone)]
pub struct SchemaArgs {
    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: OutputFormat,
}

pub fn handle_schema_cmd(root: Command, args: SchemaArgs) -> Result<()> {
    print_output(&walk(&root), args.format)?;
    Ok(())
}

fn walk(cmd: &Command) -> Value {
    let args: Vec<Value> = cmd
        .get_arguments()
        .filter(|a| a.get_id() != "help")
        .map(arg_json)
        .collect();

    let subcommands: Vec<Value> = cmd
        .get_subcommands()
        .filter(|sc| sc.get_name() != "help")
        .map(walk)
        .collect();

    json!({
        "name": cmd.get_name(),
        "about": cmd.get_about().map(|a| a.to_string()),
        "args": args,
        "subcommands": subcommands,
    })
}

fn arg_json(arg: &Arg) -> Value {
    // Set/Append actions consume values; SetTrue/SetFalse/Help/... don't.
    let takes_value = matches!(
        arg.get_action(),
        clap::ArgAction::Set | clap::ArgAction::Append
    );
    let mut v = json!({
        "id": arg.get_id().as_str(),
        "long": arg.get_long(),
        "short": arg.get_short().map(|c| c.to_string()),
        "required": arg.is_required_set(),
        "takes_value": takes_value,
    });

    if takes_value {
        let defaults: Vec<String> = arg
            .get_default_values()
            .iter()
            .map(|d| d.to_string_lossy().into_owned())
            .collect();
        if !defaults.is_empty() {
            v["default"] = json!(defaults);
        }
        let possible: Vec<String> = arg
            .get_possible_values()
            .iter()
            .map(|p| p.get_name().to_string())
            .collect();
        if !possible.is_empty() {
            v["possible_values"] = json!(possible);
        }
    }

    v
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The dumped schema must stay valid JSON (the machine-readable
    /// contract) and must contain the generated tree's leaf ids — the
    /// dotted method ids ARE the leaf names, so the dump self-documents
    /// every callable method.
    #[test]
    fn schema_dump_is_valid_json_and_covers_the_generated_tree() {
        let root = crate::cli::root_command();
        let dump = serde_json::to_string(&walk(&root)).expect("dump serializes");
        let reparsed: Value = serde_json::from_str(&dump).expect("dump re-parses as valid JSON");
        assert!(reparsed.get("name").map(|n| n == "grr").unwrap_or(false));

        for leaf in [
            "gmail.users.messages.list",
            "tasks.tasklists.list",
            "chat.spaces.get",
        ] {
            assert!(dump.contains(leaf), "dump is missing `{leaf}`");
        }
        // The static commands ride along.
        assert!(dump.contains("\"auth\""));
        assert!(dump.contains("\"api\""));
    }

    #[test]
    fn schema_dump_stays_fast_for_a_308_leaf_tree() {
        let root = crate::cli::root_command();
        let started = std::time::Instant::now();
        let dump = serde_json::to_string(&walk(&root)).expect("dump serializes");
        let elapsed = started.elapsed();
        // Generous ceiling so a slow CI box never flakes; the point is to
        // notice an order-of-magnitude regression, not to benchmark.
        assert!(
            elapsed < std::time::Duration::from_secs(1),
            "serialising the tree took {elapsed:?} for {} bytes",
            dump.len()
        );
    }
}
