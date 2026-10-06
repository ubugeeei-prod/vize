# Shared editor reference options

Tracked in [#7698](https://github.com/ubugeeei-prod/vize/issues/7698). This
preparation is private until the urgent #8116 readiness fix actually merges.
The paired issue comment is prepared locally for the owning delivery change;
it has not been published.

Native diagnostic collection adds the current workspace declaration paths to
the configured virtual TypeScript options. Canonical editor requests previously
used the base options without those paths. For a workspace with declarations,
those are different inputs to the existing projection namespace hash. That
source mechanism can select different mirror projects; it does not establish
an observed switch, a measured slowdown, or the cause of a user's latency.

Both callers now obtain one owned options value from a shared native-only
async helper. It awaits the existing generation-cached background declaration
scan, then clones the configured base options and adds the complete ordered
paths using the existing conversion. It introduces no cache or native query.
Document flags, overlays, selected project/configuration, source mappings,
request scopes, cancellation, worker deadlines, and retirement stay intact.

The authored corpus reuses two existing public inputs. Three complete-options
laws cover all declaration suffixes, dependency exclusion, repeated requests,
create/rename/delete, workspace replacement, and configured-global reload.
Existing Nuxt UI/no-tsconfig parity, global discovery/event and package
reference controls remain byte-exact and must execute on the eventual exact
source alongside whole diagnostics and hover responses.

TODO: genuinely incorporate the actual #8116 main, publish the paired issue
decision, and run the current mandatory source/native/full/protected gates.
Actual native project continuity and any latency benefit are unexecuted.
The path list is not proof that every transitive referenced declaration byte
belongs to the alias context's strong input stamps. Preserve existing watched
declaration refresh and all known strong config/package/source fingerprints;
this change grants no broader unobserved-disk-change guarantee.

No private project inputs, new pipeline stage, SDK change, oracle weakening,
instruction ceiling change, or speed/10x claim is part of this preparation.
