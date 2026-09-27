// Shared type definitions for the distilled Discovery index
// (src/discovery/*.json): the contract between fetch-discovery.ts, which
// writes the index, and generate-commands.ts, which reads it, so the two can
// never drift. Type-only on purpose — every import of this module must be
// `import type`, which Node's native type stripping (>= 23.6) erases before
// execution, so this file is never loaded at runtime.

/** One (name-sorted) Discovery parameter on a distilled method. */
export interface DistilledParameter {
  name: string;
  type: string;
  required: boolean;
  repeated: boolean;
  location: string | null;
  description: string;
  enum: Array<string | number> | null;
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
