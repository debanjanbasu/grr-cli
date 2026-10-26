// Shared type definitions for the distilled Discovery index
// (src/discovery/*.json): the contract between fetch-discovery.ts, which
// writes the index, and the generator scripts (generate-commands.ts,
// generate-skills.ts), which read it, so the two can never drift. Type-only
// on purpose — every import of this module must be `import type`, which
// Node's native type stripping (>= 23.6) erases before execution, so this
// file is never loaded at runtime.

/** One (name-sorted) Discovery parameter on a distilled method. */
export interface DistilledParameter {
  name: string;
  type: string;
  required: boolean;
  repeated: boolean;
  location: string | null;
  description: string;
  enum: Array<string | number> | null;
  /** Per-enum-value prose, aligned with `enum` by index; null when absent. */
  enumDescriptions: string[] | null;
  /** Discovery format hint (`int32`, `google-fieldmask`, `uint32`, ...). */
  format: string | null;
  /** Documented default, as Discovery spells it (always a string). */
  default: string | null;
  minimum: number | null;
  maximum: number | null;
  /** Regexp the value must match, from Discovery's `pattern`. */
  pattern: string | null;
}

/** One method, flattened to its full dotted id (`users.messages.list`). */
export interface DistilledMethod {
  id: string;
  httpMethod: string;
  path: string;
  flatPath: string | null;
  description: string;
  scopes: string[];
  required: string[];
  parameters: DistilledParameter[];
  /** `$ref` into `schemas` for the response body; null when the method
   * documents no response (or returns an empty body). */
  responseRef: string | null;
  /** `$ref` into `schemas` for the request body; null for methods whose
   * parameters fully describe the request. */
  requestRef: string | null;
}

/** A pruned JSON-schema for a response type: field names, types, array and
 * enum shapes, and required markers — deliberately WITHOUT descriptions,
 * which would multiply the index size for little agent value (the field
 * names are the payload; agents that want prose read the resource). */
export interface DistilledSchema {
  type: string;
  properties?: Record<string, DistilledSchema>;
  items?: DistilledSchema;
  enum?: Array<string | number>;
  required?: string[];
}

/** One per-service index file (src/discovery/<name>.json). */
export interface DistilledIndex {
  name: string;
  version: string;
  revision: string | null;
  title: string;
  rootUrl: string | null;
  servicePath: string;
  basePath: string;
  batchPath: string | null;
  methods: DistilledMethod[];
  /** Response schemas by name (Discovery's `schemas`, pruned). Methods'
   * `responseRef` keys into this map. */
  schemas: Record<string, DistilledSchema>;
}

/** Per-service row in src/discovery/manifest.json. */
export interface ManifestServiceEntry {
  methods: number;
  scopes: number;
  revision: string | null;
}

/** src/discovery/manifest.json — the generated-at marker the command-tree
 * generator reuses so its output stays deterministic. */
export interface Manifest {
  generatedAt: string;
  services: Record<string, ManifestServiceEntry>;
}
