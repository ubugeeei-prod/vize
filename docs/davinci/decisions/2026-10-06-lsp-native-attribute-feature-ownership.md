# Native attribute helper feature ownership

The delivered #8009 non-model component-attribute selector has one production
caller: `corsa_support/canonical/attribute_query.rs`. Its containing
`corsa_support` module is gated by the `native` feature. The helper in
`definition/helpers.rs` lacked that same gate, so unchanged NoNative builds
failed with `dead_code` under `RUSTFLAGS=-D warnings`.

Fresh full Check [37347905314](https://github.com/ubugeeei-prod/vize/actions/runs/37347905314),
Clippy job 111891086834 at `c9f0f590`, exposed the failure. The retained complete
log is 145,715 bytes with SHA256
`2732785e2930407ef04f679d4289129bcf9bdce2f2518f467c1e32afd574b47a`.
The paired issue receipt is [5999681226](https://github.com/ubugeeei-prod/vize/issues/8009#issuecomment-5999681226).

The independent correction adds only `#[cfg(feature = "native")]` to that
helper. Its body, ordinary shared definition helper and all original #8009
corpus/native laws remain unchanged. No warning suppression or forced native
feature is introduced; private #8010 writer semantics are a separate slice.

TODO: qualify this exact source with normal Actions and the existing full
feature contract (NoNative plain/glyph check and structural tests), retain
native/protected suite compatibility, actual signed merge and successor
installed release proof. No historical green is transferred.

Normal source Check 37349489989 passed at `0cd0e5d4`. Full Check 37349487690
passed its unchanged native/NoNative Clippy feature-contract step, but the
overall full envelope remains red on the existing editor code-action oracle:
both actual complete actions include diagnostics absent from its expectations.
The complete failed editor log is 246,352 bytes, SHA256
`95f563c2cc9e0b7e1b70b7f1b139d2152dbe045493964c88e0cdf4ea9c891c8e`.

The bottom #8059 genuinely replays signed actual main `7f7def6c`. The sole
editor-oracle owner branches the child from its literal frozen head, uses the
parent branch as base and registers/verifies the two PRs in an ordered native
GitHub Stack. Combined exact-head full Actions must qualify both corrections
before `gh stack merge <highest-ready-PR> --yes --squash` admits the protected
prefix. Neither layer receives individual auto-merge; the known-red full gate
is preserved. The paired decision is [5999927791](https://github.com/ubugeeei-prod/vize/issues/8009#issuecomment-5999927791).

Signed actual main later advanced to `5487072207` (#8030), and the real
combined-source merge-tree exposed a canonical-only conflict. The bottom
genuinely replays that main; the canonical is reconstructed from every exact
incoming byte plus only its owned clause. The sole child then replays its one
strict response-oracle commit onto the fresh bottom. Helper/native corpus
bytes stay unchanged; fresh source/full qualification replaces historical
credit. The paired receipt is [6000057273](https://github.com/ubugeeei-prod/vize/issues/8009#issuecomment-6000057273).

The obsolete `0cd0e5d4` full run was later terminal CANCELLED after an exact
source-head and failed-editor-job readback; only its completed feature-contract
Clippy step succeeded. The containing job and run are CANCELLED, so no overall
full/native test credit follows. The complete log is 1,109,347 bytes, SHA256
`85077bd2e0ec9b9d1702811ff01db4dd3a5d598e00b36bc7485318707c88ee5d`;
[6000271572](https://github.com/ubugeeei-prod/vize/issues/8009#issuecomment-6000271572)
retains the CAS and terminal receipt. Healthy queue/source checks were untouched.

Actual signed #8052 merged as `6f17283b88b986cf7eda38bd3cd97964a775933f` at
2026-10-05T18:17:39Z. The same bottom genuinely replays this actual source and
adds `native` gates to the moved definition in `helpers/attributes.rs` and its
`helpers.rs` reexport. The full definition body/signature and broader shared
attribute selectors stay exact; the new test consumer is inside native-gated
`hover/html`, and the sole production consumer remains native-gated Corsa.
The canonical retains every incoming main byte, moving only this owned clause
away from the shared final-line hotspot. No unmerged/dequeued tail is imported.
The paired decision is [6000519610](https://github.com/ubugeeei-prod/vize/issues/8009#issuecomment-6000519610).
Current source peer and the sole editor child's literal parent replay precede
verified native Stack registration and one combined fresh full run. Native,
NoNative plain/glyph, editor, full/protected suites, actual signed delivery and
successor installed release proof remain required; no historical green transfers.

The actual combined child `94a099ce` full Check37356286971 then failed Clippy
job111919421513 before Test: the signed #8052 span getter reexport also has only
native external consumers. The complete 147,952-byte log has SHA256
`88a08f2378d9b74e5d2e8d05ab543d9bfe31d27dfc1d5c7e629b05f88be4f32c`.
[6000677996](https://github.com/ubugeeei-prod/vize/issues/8009#issuecomment-6000677996)
pairs the narrow successor: gate only that reexport with the native non-model
selector, keeping its complete definition available to the ordinary internal
shared getter. Both Stack layers stay Draft and unqueued; strict editor oracle,
all original corpus/native laws and caps remain exact. Fresh source peer,
literal child replay and one meaningful combined full run are required before
protected-prefix admission, without native runtime credit from the failed run.

Actual signed runner repair #8066 merged as `2902dc98751b408e803ee663aeee494ffab8413b`
at 2026-10-05T21:02:06Z, with protected Check37370882169 SUCCESS. The same
bottom genuinely incorporates this literal main; helper bodies, all original
native/corpus, source author/date/body/footer and every incoming canonical byte
are exact. [6002889417](https://github.com/ubugeeei-prod/vize/issues/8009#issuecomment-6002889417)
preserves frozen440 full/source envelopes as unqualified after hosted jobs
failed acquisition, alongside their separately scoped successful execution.
The sole child follows the new literal parent, then fresh normal/source-report
and combined native/NoNative/Vue/editor full gates precede ordered native
protected-prefix admission, actual signed delivery and the next release.
No historical acceptance transfers and no individual auto-merge is enabled.
