//! Shared constants for the guard's control fixtures (#4126).
//!
//! Kept apart from the fixtures themselves because both of those register a
//! `node:test` case at import time: a test file that imported one to reach a
//! constant would adopt its case, and in the leaking fixture's case would leak
//! a child into the wrong process group.

/** How long the leaked child stays alive if nothing reaps it. */
export const LEAK_LIFETIME_MS = 60_000;

/**
 * Where the leaking fixture records the pid it abandoned.
 *
 * The file is what tells a red control apart: no file means the phase never
 * ran at all, a file whose pid the guard did not report means the guard is
 * blind. Without it, both look identical from the outside.
 */
export const LEAK_PID_FILE_ENV = "VIZE_LEAKED_CHILD_PID_FILE";
