# Shared editor reference options

Tracked in [#7698](https://github.com/ubugeeei-prod/vize/issues/7698). This
source preparation incorporates the signed #8116 readiness fix
`6b5e6fd8357bc892d2ea9cefe271a13fc08100cb` after its actual merge.
The paired issue decision is [recorded here](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6015282967).

Native diagnostic collection adds the current workspace declaration paths to
the configured virtual TypeScript options. Canonical editor requests previously
used the base options without those paths. For a workspace with declarations,
those are different inputs to the existing projection namespace hash. That
source mechanism can select different mirror projects; it does not establish
an observed switch, a measured slowdown, or the cause of a user's latency.

Both callers now obtain one owned options value from a shared native-only
async helper. It clones the configured base options before awaiting the existing
generation-cached background declaration scan, preserving the old diagnostic
read order, then adds the complete ordered paths using the existing conversion.
It introduces no cache or native query.
Document flags, overlays, selected project/configuration, source mappings,
request scopes, cancellation, worker deadlines, and retirement stay intact.

The authored options corpus reuses two existing public inputs. Three complete-options
laws cover all declaration suffixes, dependency exclusion, repeated requests,
create/rename/delete, workspace replacement, and configured-global reload.
Existing Nuxt UI/no-tsconfig parity, global discovery/event and package
reference controls remain byte-exact and must execute on the eventual exact
source alongside whole diagnostics and hover responses. A new prepared native
law uses the existing public test-package and native-runtime fixture helpers.
It compares complete Markdown hover and diagnostic vectors in six generations:
unchanged declarations, declaration-content change, rename, delete/create, an
unsaved missing-property error, and its repair. Passive test-only custody records
the successful mirror root and generated URI, bound to the exact server,
authored URI, full source, version and revision. Both features must retain one
root when paths remain fixed and choose the same new root after path changes.
The retained request scope must reject the declaration mutation. This does not
claim a process PID, native API project identity or process reaping.

TODO: run the current mandatory source/native/full/protected gates. The existing original400 workflow now
triggers on the exact changed Maestro leaves and admits only those leaves in
its closed delta set. Its recipe, fixtures, rows and budgets remain unchanged.
The first canonical request now awaits declaration discovery; its original400
first/warm cost must be measured. Actual native mirror continuity and any
latency benefit remain unexecuted. The small native law uses the existing
fixture Vue declaration package; it cannot substitute for the original full
Nuxt/Vue controls or the original400 performance comparison.
The path list is not proof that every transitive referenced declaration byte
belongs to the alias context's strong input stamps. Preserve existing watched
declaration refresh and all known strong config/package/source fingerprints;
this change grants no broader unobserved-disk-change guarantee.

No private project inputs, new pipeline stage, SDK change, oracle weakening,
instruction ceiling change, or speed/10x claim is part of this preparation.

The first published helper source at `3f46bf8` fails Clippy on eleven uncontextualized
unwraps in the prepared fixture helpers. Replace those helper-only unwraps with
contextual assertions while preserving all full input and expected vectors.
The separate actual release candidate `337dbf3` fails the unchanged CLI/editor
assembly law: CLI returns the expected three diagnostics, while the filtered
editor result is empty. The cause is not established. Test-only, test-thread
custody now records the complete pre-filter editor set, generated source and
mappings, decoded native diagnostics or errors, request stamps, full fixture
configuration, options, elapsed time and collection outcome. It adds no native
query and changes no original APP, expected diagnostic, native method, deadline,
retry, fallback or diagnostic count. Decoded bridge diagnostics are not a full
native wire or report-discriminant capture. A successful new run alone cannot
identify the earlier cause; preserve the original failed logs and require
meaningful current full/native qualification before delivery or release.
The paired decision is [6015578882](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6015578882).

The first full Vue run at `3f46bf8` also exposes an obsolete Element Plus
hover assertion. Its unchanged authored `ElBadge.content` callback accepts
`{ value: string }`; the ambient constructor mapping and existing optional-slot
factory preserve that payload. Existing broken-property CLI, VueTsc and editor
laws already require string. Correct only the complete hover Markdown literal
and its widening comment, preserving every input and other assertion. The old
full failure remains failed; current full qualification and the separate empty
assembly cause remain pending. This is an authored type contract, not a
recaptured response or performance claim. Paired decision:
[6015819899](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6015819899).
