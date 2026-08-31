// Shared primitives for the davinci bench budget gate
// (tools/davinci/bench-compare.mjs and its sibling modules here).
//
// `ConfigError` is the one error class the CLI translates into exit code 2:
// a usage or artifact-shape problem, as opposed to a budget breach (exit 1).

export const BENCH_ID = /^[A-Za-z0-9._-]+$/;

export class ConfigError extends Error {}

export function fail(message) {
  throw new ConfigError(message);
}
