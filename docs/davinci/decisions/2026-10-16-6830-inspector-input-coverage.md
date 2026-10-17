# Require every original Inspector input

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Inspector's application gate previously required only a positive reported file
count before checking difference budgets. A partial report could therefore
appear to improve parity by omitting difficult inputs.

Pin each original gitlink's source path count and SHA-256 of sorted POSIX paths
joined with LF, including the final LF. Enumerate each application's existing
source globs independently and require that pinned identity before accepting
the comparison report's exact set, once per file. Overlapping
input globs do not duplicate expectations. The summary count must also match.
An empty source fixture, missing file, repeated file, substituted file or extra
record fails. Deleting or renaming a source before both enumerations also fails.
Preserve the full report artifact before these assertions so a
failed observation remains inspectable.

This strengthens input coverage without changing selected applications, source
patterns, compiler options, error expectations or directional difference
budgets. Temporary real-file controls exercise each omitted/duplicate/substituted
input failure as well as overlapping globs, an incorrect summary count and
source deletion/rename on disk. This pins path identity; it does not replace
the existing application preparation's source-content custody checks.

The path inventories come from each pinned repository's Git tree, independently
of the Inspector report and mutable preparation directory:

| Application | Original gitlink                           | Vue files | Path-list SHA-256                                                  |
| ----------- | ------------------------------------------ | --------: | ------------------------------------------------------------------ |
| Elk         | `8aa2b4650760eeba4ea1fdd9dc18d8a88ca5d7f0` |       259 | `d02a84ea90084e448e52b595e36d740296f72f8d6f16d8f71d2c35ee5a63e6f0` |
| VueFes      | `8eff3cd7dd79a7f47bbf2b78202b9aa2070ec5a6` |        71 | `ab1162e3967372fa4f651c1cd39d4ceb37b85de84d79769ff908f6a8935a8c72` |
| npmX        | `c9e8be91e61eeeae956e2068722caa09e293cc68` |       220 | `1414645911bc91961dd8bd248a08e6cb2eec188695ae8cdbb5ae1672cb3f4bec` |

DOM and SSR are separate named test cases for each application. A DOM
assertion previously stopped the shared test before SSR ran, discarding the
second observation precisely when qualification failed. Both targets now run
and retain their complete report independently; either failure still fails
the application gate. This changes no numerical bound or error expectation.

A controlled CLI-transport probe makes DOM return a native-error mismatch
while SSR returns a complete valid input census. The original combined case
exits 1 without an SSR artifact; the separate cases retain both reports and
still exit 1. This verifies continuation and failure propagation, not compiler
behavior. The source-built App run remains the compiler authority.
