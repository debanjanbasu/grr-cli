//! Safety profiles — a cheap gate over the Discovery method surface.
//!
//! One [`SafetyProfile`] answers "may this method run?" for both dispatch
//! surfaces: the CLI (`grr api call`, and the generated tree once the
//! profile is wired through `cli.rs`) and the `grr mcp` server. The check
//! is a verb + service-name lookup per call — no I/O — so it can sit on
//! every request path.
//!
//! ```text
//! grr api call gmail.users.messages.send --readonly           # refused: POST
//! grr api call gmail.users.messages.list --deny-service chat  # allowed
//! grr mcp --readonly                                          # the server refuses writes
//! ```

use crate::discovery::Method;
use clap::Args;
use std::collections::BTreeSet;

/// Verbs that mutate server-side state. Deliberately wider than
/// `plan_request`'s body-synthesis set (POST/PATCH/PUT): DELETE mutates
/// too, it just carries no body.
pub(crate) const WRITE_VERBS: [&str; 4] = ["POST", "PATCH", "PUT", "DELETE"];

/// Is this HTTP verb a write? Shared by the gate and the MCP tool
/// descriptions so both surfaces speak the same vocabulary.
pub(crate) fn is_write_verb(verb: &str) -> bool {
    WRITE_VERBS.contains(&verb)
}

/// CLI flags for the gate, as a derive struct so `cli.rs` can flatten it
/// onto any subcommand (or the root) without re-declaring the flags.
/// `grr api call` embeds the same three fields.
#[derive(Args, Debug, Clone, Default)]
pub struct SafetyArgs {
    /// Refuse every write method (POST/PATCH/PUT/DELETE); only reads run
    #[arg(long)]
    pub readonly: bool,

    /// Refuse every method of this service (e.g. gmail). Repeatable.
    #[arg(long = "deny-service", value_name = "SERVICE")]
    pub deny_service: Vec<String>,

    /// Refuse this HTTP verb (e.g. DELETE). Repeatable.
    #[arg(long = "deny-verb", value_name = "VERB")]
    pub deny_verb: Vec<String>,
}

/// The same three flags as `global(true)` args, for attaching to the root
/// command.
///
/// Global is what makes the gate ergonomic: `--readonly` parses whether it
/// appears before the subcommand (`grr --readonly gmail …`) or after it
/// (`grr gmail users messages list --readonly`), because clap merges global
/// args across every level. `SafetyArgs::from_arg_matches` on the ROOT
/// matches then finds them wherever they were specified.
pub fn global_args() -> impl IntoIterator<Item = clap::Arg> {
    use clap::Arg;
    // The IDs MUST be the field names (`deny_service`, not `deny-service`):
    // the derive struct's from_arg_matches looks up args by the FIELD name,
    // and a hyphenated ID here is the exact "Mismatch between definition
    // and access" panic this once produced.
    [
        Arg::new("readonly")
            .long("readonly")
            .global(true)
            .action(clap::ArgAction::SetTrue)
            .help("Refuse every write method (POST/PATCH/PUT/DELETE); only reads run"),
        Arg::new("deny_service")
            .long("deny-service")
            .global(true)
            .value_name("SERVICE")
            .action(clap::ArgAction::Append)
            .help("Refuse every method of this service (repeatable)"),
        Arg::new("deny_verb")
            .long("deny-verb")
            .global(true)
            .value_name("VERB")
            .action(clap::ArgAction::Append)
            .help("Refuse this HTTP verb (repeatable)"),
    ]
}

/// The safety gate. `check` runs per method invocation on `grr api call`,
/// `gen_dispatch::dispatch_with_profile` and the `grr mcp` server — one
/// verb + service lookup, never any I/O.
#[derive(Debug, Clone, Default)]
pub struct SafetyProfile {
    /// Refuse every write verb.
    pub readonly: bool,
    /// Services whose methods are refused (matched case-insensitively).
    pub denied_services: BTreeSet<String>,
    /// Verbs that are refused (upper case on the way in).
    pub denied_verbs: BTreeSet<String>,
}

impl SafetyProfile {
    /// The default behaviour: nothing refused. A const so the permissive
    /// path needs no allocation.
    pub const PERMISSIVE: SafetyProfile = SafetyProfile {
        readonly: false,
        denied_services: BTreeSet::new(),
        denied_verbs: BTreeSet::new(),
    };

    /// A reads-only profile: every write verb is refused.
    pub fn readonly() -> Self {
        SafetyProfile {
            readonly: true,
            ..SafetyProfile::default()
        }
    }

    /// Build a profile from raw flag values, normalising on the way in:
    /// service names are matched case-insensitively (`--deny-service
    /// Gmail` == `--deny-service gmail`), verbs are upper-cased so
    /// `--deny-verb delete` refuses `DELETE`.
    pub fn new(
        readonly: bool,
        denied_services: impl IntoIterator<Item = String>,
        denied_verbs: impl IntoIterator<Item = String>,
    ) -> Self {
        SafetyProfile {
            readonly,
            denied_services: denied_services
                .into_iter()
                .map(|s| s.trim().to_ascii_lowercase())
                .collect(),
            denied_verbs: denied_verbs
                .into_iter()
                .map(|v| v.trim().to_ascii_uppercase())
                .collect(),
        }
    }

    /// From the CLI flags.
    pub fn from_args(args: &SafetyArgs) -> Self {
        SafetyProfile::new(
            args.readonly,
            args.deny_service.iter().cloned(),
            args.deny_verb.iter().cloned(),
        )
    }

    /// May this method run? `Ok(())` = allowed; `Err(message)` = refused,
    /// with an actionable message naming the method, the verb and how to
    /// lift the restriction. Precedence is readonly, then deny-service,
    /// then deny-verb — deterministic, so a method caught by two rules
    /// always reports the same one.
    pub(crate) fn check(&self, service_name: &str, method: &Method) -> Result<(), String> {
        // `check` takes the service name separately: a method id excludes
        // its service prefix (`users.messages.list` lives in `gmail`), and
        // deny-service is defined over services.
        let id = format!("{}.{}", service_name, method.id);
        let verb = method.http_method.to_ascii_uppercase();

        if self.readonly && WRITE_VERBS.contains(&verb.as_str()) {
            return Err(format!(
                "`{id}` is a {verb} (write); --readonly refuses writes. \
                 Rerun without --readonly to allow writes."
            ));
        }
        if self
            .denied_services
            .iter()
            .any(|s| s.eq_ignore_ascii_case(service_name))
        {
            return Err(format!(
                "`{id}` belongs to service `{service_name}`, which --deny-service refuses. \
                 Rerun without --deny-service {service_name} to allow it."
            ));
        }
        if self.denied_verbs.contains(&verb) {
            return Err(format!(
                "`{id}` is a {verb}; --deny-verb {verb} refuses it. \
                 Rerun without --deny-verb {verb} to allow it."
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery;

    #[test]
    fn readonly_refuses_every_write_verb_and_allows_reads() {
        let profile = SafetyProfile::readonly();

        let (service, get) = discovery::resolve("gmail.users.messages.list").unwrap();
        assert!(profile.check(&service.name, get).is_ok());

        for id in [
            "gmail.users.messages.send",  // POST
            "gmail.users.messages.trash", // POST
            "gmail.users.labels.delete",  // DELETE
        ] {
            let (service, method) = discovery::resolve(id).unwrap();
            let err = profile.check(&service.name, method).unwrap_err();
            assert!(err.contains("--readonly"), "{id}: {err}");
            assert!(err.contains("Rerun without --readonly"), "{id}: {err}");
            assert!(err.contains(&method.http_method), "{id}: {err}");
            assert!(err.contains(id), "{id}: {err}");
        }
    }

    #[test]
    fn deny_service_refuses_only_that_service() {
        let profile = SafetyProfile::new(false, ["gmail".to_owned()], []);

        let (service, method) = discovery::resolve("gmail.users.messages.list").unwrap();
        let err = profile.check(&service.name, method).unwrap_err();
        assert!(err.contains("gmail"), "{err}");
        assert!(err.contains("--deny-service"), "{err}");

        // Another service rides through untouched, reads and writes alike.
        let (service, method) = discovery::resolve("drive.files.list").unwrap();
        assert!(profile.check(&service.name, method).is_ok());
    }

    #[test]
    fn deny_service_matches_case_insensitively() {
        let profile = SafetyProfile::new(false, ["Gmail ".to_owned()], []);
        let (service, method) = discovery::resolve("gmail.users.getProfile").unwrap();
        assert!(profile.check(&service.name, method).is_err());
    }

    #[test]
    fn deny_verb_refuses_only_that_verb() {
        let profile = SafetyProfile::new(false, [], ["DELETE".to_owned()]);

        let (service, method) = discovery::resolve("gmail.users.labels.delete").unwrap();
        let err = profile.check(&service.name, method).unwrap_err();
        assert!(err.contains("--deny-verb DELETE"), "{err}");

        // A GET under the same profile is fine; a POST too.
        let (service, method) = discovery::resolve("gmail.users.messages.list").unwrap();
        assert!(profile.check(&service.name, method).is_ok());
        let (service, method) = discovery::resolve("gmail.users.messages.send").unwrap();
        assert!(profile.check(&service.name, method).is_ok());
    }

    #[test]
    fn deny_verb_is_normalized_to_upper_case() {
        let profile = SafetyProfile::new(false, [], [" post ".to_owned()]);
        let (service, method) = discovery::resolve("gmail.users.messages.send").unwrap();
        assert!(profile.check(&service.name, method).is_err());
    }

    #[test]
    fn readonly_takes_precedence_over_the_other_rules() {
        // Deterministic precedence: a write inside a denied service always
        // reports the readonly rule.
        let profile = SafetyProfile::new(true, ["gmail".to_owned()], []);
        let (service, method) = discovery::resolve("gmail.users.messages.send").unwrap();
        let err = profile.check(&service.name, method).unwrap_err();
        assert!(err.contains("--readonly"), "{err}");
        assert!(!err.contains("--deny-service"), "{err}");
    }

    #[test]
    fn permissive_is_free_of_every_restriction() {
        for id in ["gmail.users.messages.send", "gmail.users.labels.delete"] {
            let (service, method) = discovery::resolve(id).unwrap();
            assert!(
                SafetyProfile::PERMISSIVE
                    .check(&service.name, method)
                    .is_ok()
            );
        }
    }

    #[test]
    fn from_args_matches_new_directly() {
        let args = SafetyArgs {
            readonly: true,
            deny_service: vec![" Gmail ".into()],
            deny_verb: vec!["post".into()],
        };
        let from_flags = SafetyProfile::from_args(&args);
        let direct = SafetyProfile::new(true, [" Gmail ".into()], ["post".into()]);
        assert_eq!(from_flags.readonly, direct.readonly);
        assert_eq!(from_flags.denied_services, direct.denied_services);
        assert_eq!(from_flags.denied_verbs, direct.denied_verbs);
    }
}
