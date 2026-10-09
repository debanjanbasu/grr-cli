//! `grr completions <shell>`: print a shell completion script.
//!
//! The script is generated from [`crate::cli::root_command`], the same
//! composed tree `grr --help` and `grr schema` read, so every generated
//! service command, flag and alias completes with no separate source to
//! keep in sync.

use std::io::Write;

use anyhow::Result;
use clap::Command;
use clap_complete::Shell;

/// The shells whose generators handle grr's depth. Fish is deliberately
/// absent: clap_complete's fish generator stops emitting conditions below
/// two levels of subcommand nesting, and every method sits three to five
/// levels deep (`grr gmail users messages list`), so its script would offer
/// the wrong candidates rather than none.
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum CompletionShell {
    Bash,
    Elvish,
    Powershell,
    Zsh,
}

impl From<CompletionShell> for Shell {
    fn from(shell: CompletionShell) -> Self {
        match shell {
            CompletionShell::Bash => Shell::Bash,
            CompletionShell::Elvish => Shell::Elvish,
            CompletionShell::Powershell => Shell::PowerShell,
            CompletionShell::Zsh => Shell::Zsh,
        }
    }
}

#[derive(clap::Args, Debug, Clone)]
pub struct CompletionsArgs {
    /// Shell to generate the completion script for
    #[arg(value_enum)]
    pub shell: CompletionShell,
}

pub fn handle_completions_cmd(mut root: Command, args: CompletionsArgs) -> Result<()> {
    let bin_name = root.get_name().to_owned();
    // Render into memory first: clap_complete panics on a failed write, and
    // with `panic = "immediate-abort"` a closed pipe (`grr completions zsh |
    // head`) would abort instead of returning an error.
    let mut script = Vec::new();
    clap_complete::generate(Shell::from(args.shell), &mut root, bin_name, &mut script);
    std::io::stdout().lock().write_all(&script)?;
    Ok(())
}
