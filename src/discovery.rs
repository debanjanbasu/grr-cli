//! Discovery-driven API surface.
//!
//! Google's Discovery Service publishes a machine-readable description of
//! every Workspace method: HTTP verb, path template, parameters and OAuth
//! scopes. `scripts/fetch-discovery.mjs` distils those documents into a
//! compact index (method id, verb, path, scopes, parameters) which this
//! module embeds — 308 methods across ten services in ~350 KiB, versus
//! several megabytes for the raw documents.
//!
//! This is the escape hatch that makes the curated commands unnecessary for
//! coverage: `grr api call <service>.<resource>.<method>` reaches every
//! method Google publishes, including ones that did not exist when grr was
//! written, with no code change and no release.
//!
//! Two properties matter more than the coverage:
//!
//! 1. **Least privilege.** Each method carries its own scope list, and grr
//!    authorises only the scopes the requested method needs. Asking for all
//!    97 Workspace scopes up front would exceed the ~25-scope ceiling
//!    Google applies to unverified apps in testing mode, and would fail at
//!    consent; per-method authorisation sidesteps that entirely.
//! 2. **Offline.** The index is compiled in, so `grr api list` and
//!    `describe` work with no network and no cache directory.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// `(filename, service name, api version, discovery url)` — the version is
/// what callers need for display; the url is what `grr api refresh` fetches.
const SERVICES: &[(&str, &str, &str, &str)] = &[
    (
        "gmail.json",
        "gmail",
        "v1",
        "https://gmail.googleapis.com/$discovery/rest?version=v1",
    ),
    (
        "calendar.json",
        "calendar",
        "v3",
        "https://www.googleapis.com/discovery/v1/apis/calendar/v3/rest",
    ),
    (
        "drive.json",
        "drive",
        "v3",
        "https://www.googleapis.com/discovery/v1/apis/drive/v3/rest",
    ),
    (
        "people.json",
        "people",
        "v1",
        "https://people.googleapis.com/$discovery/rest?version=v1",
    ),
    (
        "chat.json",
        "chat",
        "v1",
        "https://chat.googleapis.com/$discovery/rest?version=v1",
    ),
    (
        "forms.json",
        "forms",
        "v1",
        "https://forms.googleapis.com/$discovery/rest?version=v1",
    ),
    (
        "tasks.json",
        "tasks",
        "v1",
        "https://tasks.googleapis.com/$discovery/rest?version=v1",
    ),
    (
        "docs.json",
        "docs",
        "v1",
        "https://docs.googleapis.com/$discovery/rest?version=v1",
    ),
    (
        "sheets.json",
        "sheets",
        "v4",
        "https://sheets.googleapis.com/$discovery/rest?version=v4",
    ),
    (
        "slides.json",
        "slides",
        "v1",
        "https://slides.googleapis.com/$discovery/rest?version=v1",
    ),
];

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub required: bool,
    #[serde(default)]
    pub repeated: bool,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default, rename = "enum")]
    pub enum_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct Method {
    pub id: String,
    #[serde(rename = "httpMethod")]
    pub http_method: String,
    pub path: String,
    #[serde(default)]
    pub flat_path: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
}

impl Method {
    /// Narrowest scope this method accepts, or `None` for the public
    /// methods that need no authorisation at all.
    ///
    /// Discovery lists scopes in descending breadth
    /// (`https://mail.google.com/` first), and the trailing entries are the
    /// narrow `.../auth/gmail.readonly` style. Preferring the last one keeps
    /// a `describe` call from requesting full mailbox access.
    pub fn least_privilege_scope(&self) -> Option<&str> {
        self.scopes
            .iter()
            .rev()
            .find(|s| s.starts_with("https://www.googleapis.com/auth/"))
            .map(String::as_str)
            .or_else(|| self.scopes.first().map(String::as_str))
    }

    pub fn parameter(&self, name: &str) -> Option<&Parameter> {
        self.parameters.iter().find(|p| p.name == name)
    }

    /// Placeholders in the path template, e.g. `{userId}` in
    /// `gmail/v1/users/{userId}/messages`.
    ///
    /// RFC 6570 reserved-expansion markers are stripped: `{+name}` still
    /// refers to the parameter named `name` (Chat, People and Slides write
    /// their resource-name paths that way), so the returned names line up
    /// with Discovery's parameter list.
    pub fn path_placeholders(&self) -> Vec<&str> {
        let mut out = Vec::new();
        let mut rest = self.path.as_str();
        while let Some(start) = rest.find('{') {
            let Some(end) = rest[start..].find('}') else {
                break;
            };
            let mut name = &rest[start + 1..start + end];
            if name.starts_with('+') {
                name = &name[1..];
            }
            out.push(name);
            rest = &rest[start + end + 1..];
        }
        out
    }

    /// The subset of [`Method::path_placeholders`] declared with RFC 6570
    /// reserved expansion (`{+name}`): their values are full resource names
    /// (`spaces/AAA/messages/BBB`) that must keep their slashes in the path.
    pub fn reserved_path_placeholders(&self) -> Vec<&str> {
        let mut out = Vec::new();
        let mut rest = self.path.as_str();
        while let Some(start) = rest.find('{') {
            let Some(end) = rest[start..].find('}') else {
                break;
            };
            let name = &rest[start + 1..start + end];
            if let Some(stripped) = name.strip_prefix('+') {
                out.push(stripped);
            }
            rest = &rest[start + end + 1..];
        }
        out
    }
}

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub revision: Option<String>,
    #[serde(default)]
    pub title: String,
    #[serde(default, rename = "rootUrl")]
    pub root_url: Option<String>,
    #[serde(default, rename = "servicePath")]
    pub service_path: String,
    #[serde(default, rename = "basePath")]
    pub base_path: String,
    #[serde(default, rename = "batchPath")]
    pub batch_path: Option<String>,
    pub methods: Vec<Method>,
}

impl Service {
    pub fn method(&self, id: &str) -> Option<&Method> {
        self.methods.iter().find(|m| m.id == id)
    }

    /// Path prefix shared by every method, normalised to have **no** leading
    /// slash.
    ///
    /// Discovery is inconsistent here: Gmail's `basePath` is `gmail/v1/`
    /// while Drive's is `/drive/v3/`. Since every `rootUrl` ends in `/`,
    /// concatenating naively yields `https://www.googleapis.com//drive/v3/...`
    /// — a double slash that Google answers with a 404 rather than a
    /// redirect. Strip it here, once, instead of at every call site.
    pub fn base(&self) -> String {
        let raw = if self.base_path.is_empty() {
            self.service_path.as_str()
        } else {
            self.base_path.as_str()
        };
        raw.trim_start_matches('/').to_owned()
    }
}

static SERVICES_CACHE: OnceLock<BTreeMap<&'static str, Service>> = OnceLock::new();

/// Where `grr api refresh` stores fetched indexes, and where `services()`
/// looks for a copy fresher than the embedded baseline.
///
/// `~/.grr/discovery/` — next to the config file, so uninstalling grr's
/// data directory removes it too.
pub fn cache_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|home| home.join(".grr").join("discovery"))
}

/// Google marks revisions with date-like strings (`20260923`), so a numeric
/// comparison is exact; an unparseable revision ranks as zero, which makes a
/// corrupt cache lose to any embedded baseline rather than win.
fn revision_rank(revision: &str) -> u64 {
    revision.trim().parse::<u64>().unwrap_or(0)
}

/// Every service, keyed by name: the embedded baseline overlaid with any
/// fresher on-disk cache. Parsed once, on first use.
///
/// This is the "best of both worlds": the embedded index keeps `grr api`
/// instant and offline-capable, while a cache written by `grr api refresh`
/// (or by a nightly CI run that shipped in a newer release) is preferred
/// whenever its revision is newer. No network I/O happens here — implicit
/// use must never block or fail on connectivity — so a user who never
/// refreshes still gets the compile-time baseline.
pub fn services() -> &'static BTreeMap<&'static str, Service> {
    SERVICES_CACHE.get_or_init(|| {
        let mut map: BTreeMap<&'static str, Service> = BTreeMap::new();
        for (file, name, version, _url) in SERVICES {
            let body = match *file {
                "gmail.json" => include_str!("discovery/gmail.json"),
                "calendar.json" => include_str!("discovery/calendar.json"),
                "drive.json" => include_str!("discovery/drive.json"),
                "people.json" => include_str!("discovery/people.json"),
                "chat.json" => include_str!("discovery/chat.json"),
                "forms.json" => include_str!("discovery/forms.json"),
                "tasks.json" => include_str!("discovery/tasks.json"),
                "docs.json" => include_str!("discovery/docs.json"),
                "sheets.json" => include_str!("discovery/sheets.json"),
                _ => include_str!("discovery/slides.json"),
            };
            let embedded = match serde_json::from_str::<Service>(body) {
                Ok(mut service) => {
                    // The filename is the source of truth for the name and
                    // version; trust it over whatever the document claimed so
                    // a mislabelled upstream doc cannot desync the two.
                    service.name = (*name).to_owned();
                    service.version = (*version).to_owned();
                    service
                }
                Err(error) => {
                    // A malformed embedded index is a build defect, not a
                    // runtime condition: fail loudly rather than silently
                    // dropping a service.
                    panic!("embedded discovery index {file} failed to parse: {error}");
                }
            };

            let fresher = cache_dir().and_then(|dir| {
                let path = dir.join(format!("{name}.json"));
                let Ok(body) = std::fs::read_to_string(&path) else {
                    return None;
                };
                match serde_json::from_str::<Service>(&body) {
                    // A corrupt cache must never take the index down: fall
                    // back to the embedded baseline and say so once.
                    Ok(cached)
                        if revision_rank(cached.revision.as_deref().unwrap_or(""))
                            > revision_rank(embedded.revision.as_deref().unwrap_or("")) =>
                    {
                        Some(cached)
                    }
                    Ok(_) => None,
                    Err(error) => {
                        tracing::warn!(
                            "discovery cache {} is corrupt ({error}); using the embedded index",
                            path.display()
                        );
                        None
                    }
                }
            });

            map.insert(name, fresher.unwrap_or(embedded));
        }
        map
    })
}

/// Resolve `service.resource.method` (or `service..method` for the few
/// top-level methods that have no resource).
pub fn resolve(id: &str) -> Result<(&'static Service, &'static Method), String> {
    let id = id.trim().trim_start_matches('.').replace('/', ".");
    let (service_name, method_id) = id.split_once('.').ok_or_else(|| {
        format!("`{id}` is not a method id; expected <service>.<resource>.<method>")
    })?;
    let service = services().get(service_name).ok_or_else(|| {
        let known: Vec<&str> = services().keys().copied().collect();
        format!(
            "unknown service `{service_name}`; known: {}",
            known.join(", ")
        )
    })?;
    let method = service.method(method_id).ok_or_else(|| {
        let mut ids: Vec<&str> = service.methods.iter().map(|m| m.id.as_str()).collect();
        ids.sort_unstable();
        // A near-miss is nearly always a typo in the *method* name, not the
        // resource path, so match on the last segment. Ranking matters:
        // `users.messages.list` and `users.drafts.list` are both one edit
        // from `lst`, and only the resource path tells them apart.
        let wanted = method_id.rsplit('.').next().unwrap_or(method_id);
        let mut scored: Vec<(usize, std::cmp::Reverse<usize>, &str)> = ids
            .iter()
            .copied()
            .filter_map(|candidate| {
                let actual = candidate.rsplit('.').next().unwrap_or(candidate);
                let bucket = if actual == wanted {
                    0
                } else if actual.starts_with(wanted) || wanted.starts_with(actual) {
                    1
                } else if edit_distance_at_most(actual, wanted, 1) {
                    // Limit 1, not 2: `lst` is one edit from `list` but two
                    // from `get`, and admitting distance 2 turns every short
                    // method name into a candidate.
                    2
                } else {
                    return None;
                };
                // Prefer the candidate sharing the most of the resource path
                // (Reverse: longer shared prefix sorts first).
                Some((
                    bucket,
                    std::cmp::Reverse(shared_prefix(candidate, method_id)),
                    candidate,
                ))
            })
            .collect();
        scored.sort();
        let mut near: Vec<&str> = scored
            .into_iter()
            .map(|(_, _, candidate)| candidate)
            .collect();
        near.truncate(5);
        if near.is_empty() {
            format!(
                "unknown method `{method_id}` in `{service_name}` ({} methods available)",
                service.methods.len()
            )
        } else {
            format!(
                "unknown method `{method_id}`; did you mean: {}",
                near.join(", ")
            )
        }
    })?;
    Ok((service, method))
}

/// Length of the longest common prefix.
fn shared_prefix(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(a, b)| a == b)
        .count()
}

/// Distil a raw Google Discovery document into the compact index this module
/// embeds.
///
/// This is the Rust twin of `collect()` in `scripts/fetch-discovery.mjs`,
/// which produces the committed baseline. Both must follow the same rules or
/// a refreshed cache would disagree with the embedded index in shape. The
/// rules that matter:
///   - walk `resources` ENTRIES (not values): the resource name prefixes the
///     method id;
///   - descriptions are whitespace-collapsed and truncated (the raw Gmail doc
///     alone is ~1.5 MB);
///   - required parameters are the ones `parameterOrder` names;
///   - parameters sort by name, methods sort by id — determinism keeps the
///     nightly refresh diff quiet.
pub(crate) fn distill(doc: &serde_json::Value) -> Result<Vec<Method>, String> {
    let mut out = Vec::new();
    let resources = doc
        .get("resources")
        .and_then(|r| r.as_object())
        .ok_or_else(|| "document has no resources object".to_owned())?;

    fn walk(node: &serde_json::Value, path: &str, out: &mut Vec<Method>) {
        let Some(object) = node.as_object() else {
            return;
        };
        if let Some(methods) = object.get("methods").and_then(|m| m.as_object()) {
            for (name, method) in methods {
                let id = if path.is_empty() {
                    name.clone()
                } else {
                    format!("{path}.{name}")
                };
                let http_method = method
                    .get("httpMethod")
                    .and_then(|v| v.as_str())
                    .unwrap_or("GET")
                    .to_owned();
                let path_template = method
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned();
                let flat_path = method
                    .get("flatPath")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned);
                let description = method
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                let description = description.chars().take(300).collect::<String>();
                let scopes = method
                    .get("scopes")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|s| s.as_str().map(str::to_owned))
                            .collect::<Vec<String>>()
                    })
                    .unwrap_or_default();
                let required = method
                    .get("parameterOrder")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|s| s.as_str().map(str::to_owned))
                            .collect::<Vec<String>>()
                    })
                    .unwrap_or_default();

                let mut parameters: Vec<Parameter> = method
                    .get("parameters")
                    .and_then(|p| p.as_object())
                    .map(|params| {
                        params
                            .iter()
                            .map(|(pname, p)| Parameter {
                                name: pname.clone(),
                                kind: p
                                    .get("type")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("string")
                                    .to_owned(),
                                required: required.iter().any(|r| r == pname),
                                repeated: p
                                    .get("repeated")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                                location: p
                                    .get("location")
                                    .and_then(|v| v.as_str())
                                    .map(str::to_owned),
                                description: {
                                    let text = p
                                        .get("description")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .split_whitespace()
                                        .collect::<Vec<_>>()
                                        .join(" ");
                                    text.chars().take(160).collect::<String>()
                                },
                                enum_values: p.get("enum").and_then(|v| v.as_array()).map(|a| {
                                    a.iter()
                                        .filter_map(|s| s.as_str().map(str::to_owned))
                                        .collect()
                                }),
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                parameters.sort_by(|a, b| a.name.cmp(&b.name));

                out.push(Method {
                    id,
                    http_method,
                    path: path_template,
                    flat_path,
                    description,
                    scopes,
                    required,
                    parameters,
                });
            }
        }
        if let Some(nested) = object.get("resources").and_then(|r| r.as_object()) {
            for (rname, resource) in nested {
                let child = if path.is_empty() {
                    rname.clone()
                } else {
                    format!("{path}.{rname}")
                };
                walk(resource, &child, out);
            }
        }
    }

    for (name, resource) in resources {
        walk(resource, name, &mut out);
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// Fetch fresh Discovery documents and write them to the cache directory.
///
/// The explicit, network path — `grr api refresh`. Everything else in this
/// module works offline off the embedded baseline; this is how a user (or a
/// nightly CI run) pulls the index forward between releases. On any failure
/// for one service, the embedded baseline simply stays in place for it: a
/// refresh must never leave the index WORSE than it was.
pub async fn refresh(filter: Option<&str>) -> Result<Vec<RefreshReport>, String> {
    use crate::core::http::build_http_client;

    let client = build_http_client().map_err(|e| e.to_string())?;
    let Some(dir) = cache_dir() else {
        return Err("could not find your home directory".to_owned());
    };
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let mut reports = Vec::new();
    for (file, name, version, url) in SERVICES {
        if let Some(wanted) = filter
            && !name.eq_ignore_ascii_case(wanted)
        {
            continue;
        }

        let body = client
            .get(url::Url::parse(url).map_err(|e| format!("{name}: bad discovery url: {e}"))?)
            .send()
            .await
            .map_err(|e| format!("{name}: {e}"))?
            .text()
            .await
            .map_err(|e| format!("{name}: {e}"))?;

        let doc: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| format!("{name}: discovery response is not JSON: {e}"))?;
        let revision = doc
            .get("revision")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned();

        // The embedded revision for this service, for the comparison report.
        let embedded_revision = services()
            .get(name)
            .and_then(|s| s.revision.clone())
            .unwrap_or_default();

        // A stale or failed refresh must not take the index down: only write
        // the cache when the fetched document distils cleanly AND is not
        // older than what is already in use.
        let methods = distill(&doc)?;
        if revision_rank(&revision) < revision_rank(&embedded_revision) {
            reports.push(RefreshReport {
                service: (*name).to_owned(),
                from: embedded_revision,
                to: revision,
                methods: methods.len(),
                wrote_cache: false,
                skipped_older: true,
            });
            continue;
        }

        let index = Service {
            name: (*name).to_owned(),
            version: (*version).to_owned(),
            revision: Some(revision.clone()),
            title: doc
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or(name)
                .to_owned(),
            root_url: doc
                .get("rootUrl")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            service_path: doc
                .get("servicePath")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned(),
            base_path: doc
                .get("basePath")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned(),
            batch_path: doc
                .get("batchPath")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            methods,
        };
        let json = serde_json::to_string(&index).map_err(|e| e.to_string())?;
        let path = dir.join(file);
        std::fs::write(&path, json).map_err(|e| format!("{}: {e}", path.display()))?;

        reports.push(RefreshReport {
            service: (*name).to_owned(),
            from: embedded_revision,
            to: revision,
            methods: index.methods.len(),
            wrote_cache: true,
            skipped_older: false,
        });
    }
    Ok(reports)
}

/// One service's refresh outcome, for the `grr api refresh` report.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RefreshReport {
    pub service: String,
    pub from: String,
    pub to: String,
    pub methods: usize,
    pub wrote_cache: bool,
    pub skipped_older: bool,
}

/// Levenshtein distance, abandoned as soon as it exceeds `limit`.
///
/// Method names are short, so the quadratic table is free; the early exit
/// keeps the whole 308-method scan cheap.
fn edit_distance_at_most(left: &str, right: &str, limit: usize) -> bool {
    let a: Vec<char> = left.chars().collect();
    let b: Vec<char> = right.chars().collect();
    if a.len().abs_diff(b.len()) > limit {
        return false;
    }
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        current[0] = i;
        let mut row_best = current[0];
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            current[j] = (previous[j] + 1)
                .min(current[j - 1] + 1)
                .min(previous[j - 1] + cost);
            row_best = row_best.min(current[j]);
        }
        if row_best > limit {
            return false;
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()] <= limit
}

/// Expand a discovery path template into a URL path, substituting `{...}`
/// placeholders from the supplied parameters.
///
/// Returns an error naming the missing placeholder rather than sending a
/// request with a literal `{userId}` in it — which Google answers with an
/// opaque 404.
///
/// `{+name}` placeholders (RFC 6570 reserved expansion) keep their value's
/// slashes and other path characters, per the expansion's meaning: a Chat
/// space name like `spaces/AAA` must remain a two-segment path, not a
/// percent-encoded blob.
pub fn build_path(
    service: &Service,
    method: &Method,
    params: &serde_json::Map<String, serde_json::Value>,
) -> Result<String, String> {
    let reserved = method.reserved_path_placeholders();
    let mut path = method.path.clone();
    for placeholder in method.path_placeholders() {
        let Some(value) = params.get(placeholder) else {
            let known: Vec<&str> = method
                .parameters
                .iter()
                .filter(|p| p.required)
                .map(|p| p.name.as_str())
                .collect();
            return Err(format!(
                "`{}` needs the `{placeholder}` parameter (path). Required: {}",
                method.id,
                if known.is_empty() {
                    "none".to_owned()
                } else {
                    known.join(", ")
                }
            ));
        };
        // Path values must be percent-encoded: ids legitimately contain
        // spaces, slashes and unicode (Drive file names, Gmail labels).
        // Reserved-expansion values are the documented exception — their
        // `/` separators ARE the path structure.
        let text = value_to_string(value);
        let encoded = if reserved.contains(&placeholder) {
            encode_path_reserved(&text)
        } else {
            urlencoding::encode(&text).into_owned()
        };
        path = path
            .replace(&format!("{{{placeholder}}}"), &encoded)
            .replace(&format!("{{+{placeholder}}}"), &encoded);
    }
    Ok(format!(
        "{}{}",
        service.base(),
        path.trim_start_matches('/')
    ))
}

/// Percent-encode a path value for RFC 6570 reserved expansion: keep every
/// `pchar` (path character) — most importantly `/`, so Google's full
/// resource names (`spaces/AAA/messages/BBB`) stay a real path — and
/// encode the rest (spaces, `?`, `#`, `%`, non-ASCII, …).
fn encode_path_reserved(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'.'
            | b'_'
            | b'~'
            | b'/'
            | b':'
            | b'@'
            | b'!'
            | b'$'
            | b'&'
            | b'\''
            | b'('
            | b')'
            | b'*'
            | b'+'
            | b','
            | b';'
            | b'=' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

pub fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Render a parameter map as a query string, skipping path parameters
/// (already substituted) and honouring `repeated: true` for multi-valued
/// params such as Gmail's `labelIds`.
pub fn build_query(method: &Method, params: &serde_json::Map<String, serde_json::Value>) -> String {
    let placeholders = method.path_placeholders();
    let mut pairs: Vec<(String, String)> = Vec::new();
    for (key, value) in params {
        if placeholders.contains(&key.as_str()) {
            continue;
        }
        if let Some(param) = method.parameter(key)
            && param.location.as_deref() == Some("path")
        {
            continue;
        }
        match value {
            serde_json::Value::Array(items) => {
                for item in items {
                    pairs.push((key.clone(), value_to_string(item)));
                }
            }
            serde_json::Value::Null => {}
            other => pairs.push((key.clone(), value_to_string(other))),
        }
    }
    if pairs.is_empty() {
        return String::new();
    }
    pairs.sort();
    let query = pairs
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    format!("?{query}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_service_parses_and_has_methods() {
        let all = services();
        assert_eq!(all.len(), SERVICES.len());
        for (name, service) in all {
            assert!(!service.methods.is_empty(), "{name} has no methods");
        }
    }

    #[test]
    fn the_index_covers_the_whole_workspace_surface() {
        // The point of the dynamic path is coverage. If Google's docs change
        // shape and we silently parse zero methods, this is what catches it.
        let total: usize = services().values().map(|s| s.methods.len()).sum();
        assert!(
            total >= 300,
            "expected 300+ methods across the Workspace APIs, found {total}"
        );
    }

    #[test]
    fn resolves_a_nested_method_id() {
        let (service, method) =
            resolve("gmail.users.messages.list").expect("gmail.users.messages.list");
        assert_eq!(service.name, "gmail");
        assert_eq!(method.http_method, "GET");
        assert_eq!(method.path, "gmail/v1/users/{userId}/messages");
    }

    #[test]
    fn resolves_a_method_without_an_intermediate_resource() {
        // `users.getProfile` sits directly on the users resource, so there is
        // only one dot before the method name.
        let (service, method) = resolve("gmail.users.getProfile").expect("gmail.users.getProfile");
        assert_eq!(service.name, "gmail");
        assert_eq!(method.http_method, "GET");
    }

    #[test]
    fn accepts_a_leading_dot_and_slashes() {
        assert!(resolve(".gmail.users.messages.list").is_ok());
        assert!(resolve("gmail/users/messages/list").is_ok());
    }

    #[test]
    fn unknown_service_and_method_are_actionable() {
        // A typo close to a real method gets a concrete suggestion.
        let err = resolve("gmail.users.messages.lst").unwrap_err();
        assert!(err.contains("did you mean"), "unhelpful: {err}");
        let err = resolve("nope.thing.method").unwrap_err();
        assert!(err.contains("known:"), "should list services: {err}");
        let err = resolve("gmailonly").unwrap_err();
        assert!(err.contains("expected <service>"), "unhelpful: {err}");
    }

    #[test]
    fn least_privilege_scope_prefers_the_narrow_one() {
        let (_, method) = resolve("gmail.users.messages.list").unwrap();
        // gmail.users.messages.list accepts full-mailbox access first in the
        // discovery list; we must not pick that.
        let scope = method.least_privilege_scope().expect("a scope");
        assert!(scope.ends_with("gmail.readonly"), "picked {scope}");
    }

    #[test]
    fn required_parameters_are_flagged() {
        let (_, method) = resolve("gmail.users.messages.list").unwrap();
        let user_id = method.parameter("userId").expect("userId param");
        assert!(user_id.required);
        let max = method.parameter("maxResults").expect("maxResults param");
        assert!(!max.required);
    }

    #[test]
    fn builds_a_path_and_reports_missing_placeholders() {
        let (service, method) = resolve("gmail.users.messages.get").unwrap();
        let mut params = serde_json::Map::new();
        params.insert("id".into(), serde_json::json!("m1"));

        // A missing path parameter must name itself rather than produce a
        // request with a literal brace in the URL.
        let err = build_path(service, method, &params).unwrap_err();
        assert!(err.contains("userId"), "unhelpful: {err}");

        params.insert("userId".into(), serde_json::json!("me"));
        assert_eq!(
            build_path(service, method, &params).unwrap(),
            "gmail/v1/users/me/messages/m1"
        );
    }

    #[test]
    fn path_values_are_percent_encoded() {
        let (service, method) = resolve("drive.files.get").unwrap();
        let mut params = serde_json::Map::new();
        params.insert("fileId".into(), serde_json::json!("a b/c+d"));
        let path = build_path(service, method, &params).unwrap();
        assert!(!path.contains(' '), "space leaked into {path}");
        assert!(path.contains("a%20b%2Fc%2Bd"), "not encoded: {path}");
    }

    #[test]
    fn query_excludes_path_params_and_expands_arrays() {
        let (_, method) = resolve("gmail.users.messages.list").unwrap();
        let mut params = serde_json::Map::new();
        params.insert("userId".into(), serde_json::json!("me"));
        params.insert("q".into(), serde_json::json!("is:unread"));
        params.insert("labelIds".into(), serde_json::json!(["INBOX", "UNREAD"]));

        let query = build_query(method, &params);
        assert!(!query.contains("userId"), "path param leaked: {query}");
        assert!(query.contains("q=is%3Aunread"), "{query}");
        assert!(query.contains("labelIds=INBOX"), "{query}");
        assert!(query.contains("labelIds=UNREAD"), "{query}");
    }

    #[test]
    fn query_is_empty_when_only_path_params_are_given() {
        let (_, method) = resolve("gmail.users.messages.list").unwrap();
        let mut params = serde_json::Map::new();
        params.insert("userId".into(), serde_json::json!("me"));
        assert!(build_query(method, &params).is_empty());
    }

    #[test]
    fn reserved_expansion_placeholders_match_their_parameters() {
        // Chat writes paths as `v1/{+name}` while the parameter is `name`:
        // the placeholder must resolve by its real name, not `+name`.
        let (service, method) = resolve("chat.spaces.get").unwrap();
        assert_eq!(method.path_placeholders(), ["name"]);
        assert_eq!(method.reserved_path_placeholders(), ["name"]);

        let mut params = serde_json::Map::new();
        params.insert("name".into(), serde_json::json!("spaces/AAA"));
        let path = build_path(service, method, &params).unwrap();
        // The slash IS the path structure: reserved expansion keeps it.
        assert_eq!(path, "v1/spaces/AAA");
    }

    #[test]
    fn reserved_expansion_only_encodes_non_path_characters() {
        let (service, method) = resolve("chat.spaces.get").unwrap();
        let mut params = serde_json::Map::new();
        params.insert("name".into(), serde_json::json!("spaces/A B?C"));
        let path = build_path(service, method, &params).unwrap();
        assert_eq!(path, "v1/spaces/A%20B%3FC");
    }

    #[test]
    fn reserved_expansion_placeholders_do_not_leak_into_the_query() {
        // `name` was consumed by the path, so it must not also appear as a
        // query pair (a stale `{+name}` key used to hide it from the check).
        let (_, method) = resolve("chat.spaces.get").unwrap();
        let mut params = serde_json::Map::new();
        params.insert("name".into(), serde_json::json!("spaces/AAA"));
        assert!(build_query(method, &params).is_empty());
    }

    #[test]
    fn plain_placeholders_still_percent_encode_slashes() {
        // Non-reserved placeholders keep the strict component encoding:
        // a Drive-style id with a slash must never smuggle in extra path
        // segments.
        let (service, method) = resolve("gmail.users.messages.get").unwrap();
        let mut params = serde_json::Map::new();
        params.insert("userId".into(), serde_json::json!("me"));
        params.insert("id".into(), serde_json::json!("a/b"));
        assert!(
            build_path(service, method, &params)
                .unwrap()
                .ends_with("a%2Fb")
        );
    }

    #[test]
    fn every_service_base_has_no_leading_slash() {
        // rootUrl always ends in '/', so a leading slash on basePath yields
        // '//' and a 404 from Google. Gmail and Drive disagree here, which is
        // exactly why this is asserted rather than assumed.
        for (name, service) in services() {
            assert!(
                !service.base().starts_with('/'),
                "{name} base starts with a slash: {:?}",
                service.base()
            );
            let root = service.root_url.clone().unwrap_or_default();
            let sample = format!("{}{}", root, service.base());
            // Skip the scheme: "https://" is a legitimate double slash.
            let after_scheme = sample
                .split_once("://")
                .map(|(_, rest)| rest)
                .unwrap_or(&sample);
            assert!(
                !after_scheme.starts_with('/'),
                "{name} would build {sample}"
            );
        }
    }

    #[test]
    fn method_ids_include_their_resource_path() {
        // Regression: the distiller once dropped the root resource name,
        // producing `messages.list` instead of `users.messages.list`.
        let (service, method) = resolve("gmail.users.messages.list").unwrap();
        assert_eq!(method.id, "users.messages.list");
        assert!(service.methods.iter().any(|m| m.id == "users.threads.get"));
    }

    #[test]
    fn edit_distance_gate_behaves() {
        assert!(edit_distance_at_most("list", "lst", 2));
        assert!(edit_distance_at_most("files", "file", 2));
        assert!(!edit_distance_at_most("list", "trash", 2));
        // Length guard must reject before allocating.
        assert!(!edit_distance_at_most("a", "abcdefghij", 2));
    }

    #[test]
    fn a_one_character_typo_ranks_the_real_method_first() {
        let err = resolve("gmail.users.messages.lst").unwrap_err();
        // Alphabetical ordering used to push the answer past the 5-item cut.
        let first = err
            .split("did you mean:")
            .nth(1)
            .expect("suggestion list")
            .trim()
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .trim_end_matches('.');
        assert!(
            first.contains("users.messages.list"),
            "best suggestion was `{first}` in: {err}"
        );
    }

    #[test]
    fn most_methods_need_only_a_handful_of_scopes() {
        // This is why per-method authorisation is viable. Asking for the
        // union of everything would need 97 scopes and blow past the ~25
        // Google allows on an unverified app; asking for one method's scopes
        // needs a median of 3.
        let counts: Vec<usize> = services()
            .values()
            .flat_map(|s| s.methods.iter())
            .map(|m| m.scopes.len())
            .collect();
        let total = counts.len();
        let within_nine = counts.iter().filter(|c| **c <= 9).count();
        assert!(
            within_nine * 100 / total >= 90,
            "only {within_nine}/{total} methods need <=9 scopes"
        );
        // A handful of endpoints genuinely accept many (chat.spaceEvents.list
        // takes 16); the point is the tail, not the ceiling.
        let worst = counts.iter().max().copied().unwrap_or(0);
        assert!(worst <= 20, "a method accepts {worst} scopes");
    }

    #[test]
    fn no_method_requests_the_full_workspace_surface_at_once() {
        // The reason per-method scopes exist: the union of every method's
        // scopes is far past what an unverified app may request at consent.
        let union: std::collections::BTreeSet<&str> = services()
            .values()
            .flat_map(|s| s.methods.iter())
            .flat_map(|m| m.scopes.iter().map(String::as_str))
            .collect();
        assert!(
            union.len() > 25,
            "expected the union to exceed Google's unverified-app ceiling, got {}",
            union.len()
        );
    }
}
