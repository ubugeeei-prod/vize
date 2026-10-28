//! TypeScript's implicit exclusion of package folders from wildcard `include`
//! segments.
//!
//! `tsc` does not implement the `node_modules` / `bower_components` /
//! `jspm_packages` default as an `exclude` entry. It bakes the exclusion into
//! how an `include` pattern is compiled: every segment that carries a wildcard —
//! and every directory a `**` consumes — is prefixed with
//! `(?!(node_modules|bower_components|jspm_packages)(/|$))`
//! (`implicitExcludePathRegexPattern`), while a literal segment is matched
//! literally. Three consequences, each verified against
//! `tsgo -p tsconfig.json --listFiles`:
//!
//! * The exclusion applies at *every* depth, not only directly below the
//!   tsconfig directory. `include: ["packages/**/*.ts"]` never collects
//!   `packages/a/node_modules/dep/index.ts`.
//! * It is not the `exclude` field, so an explicit `exclude: []` does not switch
//!   it off.
//! * Spelling a package folder out as a literal segment keeps it:
//!   `include: ["packages/*/node_modules/dep/*.ts"]` does collect that file.
//!
//! Anchoring the three directory names at the tsconfig directory only — the
//! shape [`super::glob::default_exclude_specs`] produces — therefore misses
//! nested copies, which is how files from a real nested `node_modules` became
//! program roots (#3385).
//!
//! [`vize_canon::batch::TsconfigWildcard`] compiles every spec with these rules.

/// `commonPackageFolders` in TypeScript's `utilities.ts`.
const COMMON_PACKAGE_FOLDERS: [&str; 3] = ["node_modules", "bower_components", "jspm_packages"];

pub(super) fn is_package_folder(segment: &str) -> bool {
    COMMON_PACKAGE_FOLDERS.contains(&segment)
}
