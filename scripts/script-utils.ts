// Runtime helpers shared by the generator scripts. Type-only contracts live
// in ./discovery-index.ts (which Node's type stripping erases); this module
// is loaded at runtime, so it holds only plain functions and constants.
//
// `kebab`/`flagId`/`RESERVED_FLAG_IDS` MUST agree across every generator and
// with `gen_dispatch::flag_id` in src/commands/gen_dispatch.rs: a reserved
// Discovery parameter is renamed the same way wherever a flag id is derived,
// so the command tree and the skills never disagree about a flag's spelling.

/** Escape-hatch flag ids present on every generated leaf (plus clap's own
 * `help`). A Discovery parameter whose kebab-case name lands here is renamed
 * `param-<kebab>`. */
export const RESERVED_FLAG_IDS: Record<string, true> = {
  params: true,
  'body-file': true,
  query: true,
  'dry-run': true,
  format: true,
  help: true,
};

/** camelCase (or dotted, or $-prefixed) Discovery name -> kebab-case flag. */
export function kebab(name: string): string {
  return name
    .replace(/([a-z0-9])([A-Z])/g, '$1-$2')
    .replace(/[^a-zA-Z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .toLowerCase();
}

export function flagId(name: string): string {
  const s = kebab(name);
  return RESERVED_FLAG_IDS[s] === true ? `param-${s}` : s;
}
