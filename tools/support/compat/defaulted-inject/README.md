# Explicit optional-inject transition

Issue #10918 treats an explicit inject default as intentional optionality. The production change only admits unmatched-inject findings for calls without defaults. Provider branches, tree counters, type mismatches and reactive-destructuring diagnostics remain inspected by native tests.

The historical complete snapshot and original tree test are archived byte-for-byte. The snapshot transition test permits only the single historical optional-fallback warning to disappear. The original issue body drives nine paired CLI cases against public 0.602.0; all remaining full diagnostic fields and counts must match. Complete doctor packets are captured; controls with no removed finding must match exactly. Changed doctor scores are not claimed qualified by a broad mask.

Actions must pass before merge; public-after acceptance is pending the first containing release.
