# Relative sibling storage (#10934)

Reported by dmnlk. Explicit checks import a sibling SFC, whose exported type
reaches back through an app model and a sibling JSON module. An inherited
configuration also registers a hoisted Vue JSX declaration. Public 0.596.0
reports the correct TS2322 but discards the entire materialized namespace on exit.

The checker keeps the semantic workspace root so native package resolution
retains the authored ancestor chain. The requested app owns a separate persistent
private namespace. Root/storage identity, unchanged mirror mtimes and retained
build info are checked by three fresh CLI processes: invalid, unchanged, repair.
The pinned batch cases compare all diagnostics, including exact UTF16 start,
code, severity and message. JSON-derived number types prevent silent any escape.

The native package-visibility control remains unchanged: an external sibling
cannot import an app-only package. Linked-prefix and hardlink tests preserve the
original installed bytes and mtimes. This is a storage/persistence regression,
not a claim about 2500-SFC cold latency.
