#![cfg(feature = "cli")]
//! CLI integration tests: exercise the generated command surface.
//!
//! All tests use --help / clap failures only, so they need no config
//! file, network, or credential — except the dry-run parity pair, which
//! injects a dummy OAuth client via environment variables (the config
//! loader merges `GRR_OAUTH__*` over any file) and never reaches the
//! network because --dry-run returns before a token is requested.

use assert_cmd::Command;
use predicates::prelude::*;

fn grr() -> Command {
    Command::cargo_bin("grr").unwrap()
}

/// A command with a dummy OAuth client injected so config loading and
/// auth-handle construction succeed without touching the network.
///
/// `GRR_CONFIG_PATH` points at a file that does not exist: the developer
/// machine running these tests has a real `~/.grr/config.toml`, and
/// figment rejects an env override landing on the same OAuth field the
/// file already fills (two aliases of one serde field). The missing file
/// is figment's "empty profile", so the test stays hermetic everywhere.
fn configured_grr() -> Command {
    let mut cmd = grr();
    cmd.env("GRR_CONFIG_PATH", "no-such-config-for-tests.toml")
        .env("GRR_OAUTH__CLIENT_ID", "test-id.apps.googleusercontent.com")
        .env("GRR_OAUTH__CLIENT_SECRET", "test-secret");
    cmd
}

#[test]
fn version_output_is_a_banner_with_the_semver_on_the_first_line() {
    // `--version` prints a banner: `grr <semver>`, the mascot, then the
    // tagline. The first line must stay exactly `grr <semver>` —
    // scripts/benchmark.ts parses it as the measured version and its gate
    // compares it against Cargo.toml, and generate-demo.ts matches
    // /^grr \d+\.\d+\.\d+$/m — while the Homebrew formula only needs the
    // version as a substring anywhere in the output.
    //
    // assert_cmd captures stdout through a pipe, so this also pins the
    // no-TTY contract: the plain ASCII crab, never escape sequences.
    grr()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with(concat!(
            "grr ",
            env!("CARGO_PKG_VERSION"),
            "\n"
        )))
        // the mascot rides along: wide-set glossy eyes, small smile
        .stdout(predicate::str::contains("(o) (o)"))
        .stdout(predicate::str::contains("\\_/"))
        // the tagline is the crate's about line — factual, no marketing
        .stdout(predicate::str::contains(
            "Google tools from the terminal, at maximum performance",
        ))
        .stdout(predicate::str::contains("\u{1b}").not());
}

#[test]
fn version_draws_the_pixel_mascot_when_the_terminal_advertises_colour() {
    // `CLICOLOR_FORCE` is the opt-in that asks for the full logo through a
    // pipe. The banner must still lead with the bare semver — the colour
    // logo goes below it, never in front of it.
    let output = grr()
        .env_remove("NO_COLOR")
        .env("CLICOLOR_FORCE", "1")
        .arg("--version")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let banner = String::from_utf8(output).expect("banner is UTF-8");
    let first_line = banner.lines().next().expect("banner has a first line");
    assert_eq!(first_line, format!("grr {}", env!("CARGO_PKG_VERSION")));
    // U+2580 UPPER HALF BLOCK, carrying two pixel rows per character row.
    assert!(banner.contains('\u{2580}'));
    assert!(banner.contains("\u{1b}[38;2;"), "no truecolor foreground");
    assert!(banner.contains("\u{1b}[48;2;"), "no truecolor background");
    assert!(
        banner.contains("Google tools from the terminal, at maximum performance"),
        "the colour path drops the tagline"
    );
    // Every logo row is reset, so the logo cannot bleed into the tagline.
    assert!(banner.contains("\u{2580}\u{1b}[0m\n"));
}

#[test]
fn no_color_beats_the_force_opt_in() {
    // NO_COLOR's whole contract is "never colour my output"; an
    // environment setting both has said so twice, and NO_COLOR wins.
    grr()
        .env("NO_COLOR", "1")
        .env("CLICOLOR_FORCE", "1")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("(o) (o)"))
        .stdout(predicate::str::contains("\u{2580}").not());
}

#[test]
fn top_level_help_lists_static_commands_and_every_service() {
    grr()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("api"))
        .stdout(predicate::str::contains("schema"))
        .stdout(predicate::str::contains("transport"))
        .stdout(predicate::str::contains("gmail"))
        .stdout(predicate::str::contains("calendar"))
        .stdout(predicate::str::contains("drive"))
        .stdout(predicate::str::contains("people"))
        .stdout(predicate::str::contains("chat"))
        .stdout(predicate::str::contains("forms"))
        .stdout(predicate::str::contains("tasks"))
        .stdout(predicate::str::contains("docs"))
        .stdout(predicate::str::contains("sheets"))
        .stdout(predicate::str::contains("slides"));
}

#[test]
fn gmail_help_shows_the_resource_tree() {
    grr()
        .arg("gmail")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("users"))
        .stdout(predicate::str::contains("Gmail API operations"));
}

#[test]
fn gmail_users_messages_lists_leaf_commands() {
    grr()
        .arg("gmail")
        .arg("users")
        .arg("messages")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("gmail.users.messages.list"))
        .stdout(predicate::str::contains("gmail.users.messages.get"))
        .stdout(predicate::str::contains("gmail.users.messages.send"));
}

#[test]
fn message_list_help_documents_typed_flags() {
    grr()
        .arg("gmail")
        .arg("users")
        .arg("messages")
        .arg("list")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--user-id <USER_ID>"))
        .stdout(predicate::str::contains("--max-results <MAX_RESULTS>"))
        .stdout(predicate::str::contains("--label-ids <LABEL_IDS>"))
        .stdout(predicate::str::contains("--q <Q>"))
        // required flags are promoted into the usage line by clap
        .stdout(predicate::str::contains(
            "gmail.users.messages.list [OPTIONS] --user-id <USER_ID>",
        ))
        // the escape hatches ride on every leaf
        .stdout(predicate::str::contains("--params <JSON>"))
        .stdout(predicate::str::contains("--body-file <PATH|->"))
        .stdout(predicate::str::contains("--query <KEY=VALUE>"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("-f, --format <FORMAT>"));
}

#[test]
fn a_leaf_rejects_a_missing_required_flag() {
    grr()
        .arg("gmail")
        .arg("users")
        .arg("messages")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--user-id"));
}

#[test]
fn a_leaf_rejects_unknown_flags() {
    grr()
        .arg("gmail")
        .arg("users")
        .arg("messages")
        .arg("list")
        .arg("--user-id")
        .arg("me")
        .arg("--bogus")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn format_rejection_is_a_clap_error() {
    grr()
        .arg("gmail")
        .arg("users")
        .arg("messages")
        .arg("list")
        .arg("--user-id")
        .arg("me")
        .arg("--format")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn enum_typed_flags_reject_values_outside_the_documented_set() {
    grr()
        .arg("gmail")
        .arg("users")
        .arg("messages")
        .arg("get")
        .arg("--user-id")
        .arg("me")
        .arg("--id")
        .arg("m1")
        .arg("--param-format")
        .arg("not-a-format")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn bare_gmail_without_subcommand_is_a_usage_error() {
    grr().arg("gmail").assert().failure();
}

#[test]
fn auth_help_lists_login_and_status() {
    grr()
        .arg("auth")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("login"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("setup"))
        .stdout(predicate::str::contains("--device"));
}

#[test]
fn schema_dumps_json_without_config() {
    // Must succeed with no ~/.grr config: pure contract introspection.
    grr()
        .arg("schema")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\":\"grr\""))
        .stdout(predicate::str::contains("\"subcommands\""))
        .stdout(predicate::str::contains("gmail.users.messages.list"))
        .stdout(predicate::str::contains("gmail"));
}

#[test]
fn schema_dumps_every_service() {
    grr()
        .arg("schema")
        .assert()
        .success()
        .stdout(predicate::str::contains("gmail"))
        .stdout(predicate::str::contains("calendar"))
        .stdout(predicate::str::contains("drive"))
        .stdout(predicate::str::contains("people"))
        .stdout(predicate::str::contains("chat"))
        .stdout(predicate::str::contains("forms"))
        .stdout(predicate::str::contains("tasks"))
        .stdout(predicate::str::contains("docs"))
        .stdout(predicate::str::contains("sheets"))
        .stdout(predicate::str::contains("slides"));
}

#[test]
fn schema_table_output_renders_field_value_table() {
    // Schema output is a single JSON object, so -f table takes the
    // Field/Value (non-array) table path. Array flattening is covered by
    // the unit tests in output.rs (arrays need live API results).
    grr()
        .arg("schema")
        .arg("-f")
        .arg("table")
        .assert()
        .success()
        .stdout(predicate::str::contains("Field"))
        .stdout(predicate::str::contains("Value"))
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("subcommands"));
}

#[test]
fn a_service_with_no_curated_history_is_reachable() {
    // Tasks never had hand-written commands; the generated tree is its
    // entire surface.
    grr()
        .arg("tasks")
        .arg("tasklists")
        .arg("list")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("tasks.tasklists.list"));
}

#[test]
fn the_generated_tree_and_api_call_print_identical_dry_runs() {
    // End-to-end parity: the same method and parameters through the
    // generated tree (typed flags) and through `grr api call` (--param)
    // must produce byte-identical dry-run output. Both stop before any
    // token use, so the dummy client is enough.
    let tree = configured_grr()
        .arg("gmail")
        .arg("users")
        .arg("messages")
        .arg("list")
        .arg("--user-id")
        .arg("me")
        .arg("--dry-run")
        .assert()
        .success();
    let tree_output = String::from_utf8_lossy(&tree.get_output().stdout).into_owned();

    let api = configured_grr()
        .arg("api")
        .arg("call")
        .arg("gmail.users.messages.list")
        .arg("--param")
        .arg("userId=me")
        .arg("--dry-run")
        .assert()
        .success();
    let api_output = String::from_utf8_lossy(&api.get_output().stdout).into_owned();

    assert_eq!(
        tree_output, api_output,
        "dry-run diverged between the generated tree and `grr api call`"
    );
    assert!(tree_output.contains("\"dryRun\":true"), "in: {tree_output}");
    assert!(
        tree_output.contains("https://gmail.googleapis.com/gmail/v1/users/me/messages"),
        "in: {tree_output}"
    );
}
