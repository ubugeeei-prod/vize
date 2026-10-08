# Native LSP project replacement ownership (#3952)

The current-source Misskey baseline [run 37731827784](https://github.com/ubugeeei-prod/vize/actions/runs/37731827784)
built the exact `d0d0c7db0db537c3cbc6735aa0d8f25296e26166` source. Its binary SHA256 is
`f5f63ac9e2d9331c81c657e42f7d140d341f4f5aa914e5e19907f53c3dd54851`. Artifact
`11529494144` has complete launch receipts, raw wire and stderr bytes, continuous
50 ms process/RSS observations, and all three original 40-cycle sessions. The
locally downloaded ZIP SHA256 is
`8c875a88135622fb7d4bedc328913aa7b93906ab5135c641923a43dfb3770dcc`.

All three sessions exceeded the unchanged process-tree limit of 3. The extra
process was a genuine, positive-RSS, sleeping native `tsc` child; it was neither
a wrapper nor a zombie. Every session had zero shutdown survivors.

| Session | Server PID | Transient API owners | Retained editor PID | Observed overlap after spawn |
| ------- | ---------- | -------------------- | ------------------- | ---------------------------- |
| 1       | 26256      | 26313, 26394         | 26353               | 1.242 s                      |
| 2       | 26746      | 26799, 26880         | 26839               | 1.246 s                      |
| 3       | 27197      | 27250, 27331         | 27290               | 1.248 s                      |
| 3       | 27197      | 27331, 27462         | 27290               | 13.765 s                     |

`CorsaProjectClient::activate_workspace_project_with_reload` constructs the
replacement `ProjectSession` before closing its previous API owner. Its first
root/topology changes occur while initial diagnostics prepare `MkCodeInline.vue`
and `MkDivider.vue`. The later overlap occurs when the first diagnostic for
`__vize_ephemeral_lifecycle.vue` introduces new project topology. The reusable
`EditorLspSession` remains alive during each API replacement. These three native
children plus the Vize process produce a tree of 4. Module-link context guards
and the authored diagnosing owner are separate lifecycles; this evidence does
not attribute the overlap to either.

The materialized-project transition in `bootstrap.rs` has the same
spawn-before-close ordering. Both transition paths must close and reap their
previous API process before spawning its replacement, and propagate a cleanup
failure before authorizing another process. Content-only refreshes continue to
reuse the existing API/editor owners.

The regression fixture under
`tests/_fixtures/differential/lsp/native-project-retirement-3952/` observes the
previous physical native PID, birth tick, executable and role at the actual
replacement launch boundary before execing the unchanged native binary. It
compares the complete native diagnostic report around three same-root topology
reloads and exercises a materialized-mode transition with a live editor owner.
A zombie receives no retirement credit. The existing native-backend Actions lane runs this one producer law with its
already-installed pinned native runtime and source recipe. The raw log must
show the exact named test and `running 1 test`; its source-bound receipt matches
all five launch PID/birth tuples to observed live native API executables. It
also archives all four actual complete diagnostic reports and checks each
against the original fixture. This
adds no second runtime installation to the PR Rust builder. Full workspace
merge tests also retain the law. Historical immutable-source recipe mode skips
this new observer and records `not-qualified`; it does not demand a test from
an older source or award process-fix credit to a zero-test selection. The marked
inline source recipe and original 7/9/10/11 target fixtures remain unchanged.

This slice does not close #3952. Session 2 independently failed the original
128 MiB server RSS ceiling (128.94 MiB at `file-lifecycle-1`). Source acceptance
requires the fixed source to pass all three original sessions, all 40 cycles,
the unchanged process/RSS limits and complete authored feature/lifecycle
oracles. The full 134-project acceptance and publication remain separate.

Before/after Actions evidence will be appended after terminal qualification.
