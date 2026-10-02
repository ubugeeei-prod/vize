# Native SFC publishable dependency closure

The explicit native SFC adapter introduces a normal dependency from the existing
publishable `vize_atelier_sfc` crate to `vize_l4`. Required Actions on product
source `d6f56c09` rejected that graph because L4 still declared `publish = false`
and the ordered release list omitted it. The original failed run remains a
failure; passing runtime and performance evidence does not satisfy this contract.

For the already authorized opt-in adapter, L4 becomes registry-eligible while
retaining its experimental stability metadata. Remove only its `publish = false`
entry. Its normal workspace dependencies, L0, L2 and L3, already appear in the
release plan. Add L4 immediately after L3 and before its SFC consumer in the
existing `published_crates` constant. Publisher algorithms, dependency assertions
and the exact partition test remain unchanged.

This expressly supersedes the initial L4 skeleton decision to keep the crate
unpublished. It establishes a consistent future package graph for the additive
entry. It performs no registry publication, release, Trusted Publishing setup or
default consumer replacement. First publication and release acceptance still
require their actual publishing controls and terminal external evidence.

The canonical Rust support table and L4 README document the same experimental
audience, entrypoints and deprecation contract. The README links that checked
table; existing experimental Rust documentation and metadata remain intact.

All L4 production source, APIs, fixtures, maps and runtime expectations remain
unchanged. The source-owned SFC/File custody and bounded scriptless/static/literal
family remain unchanged. Compiler fix-history issue #6880, unfinished L4 issue
#6840 and all five product route gates remain open.

Validate the actual release dependency order and exact publishable partition,
locked no-deps Cargo metadata, current migration and generated ledgers, storage
and dependency policies, source length, formatting and unchanged source hashes.
Fresh exact-head Actions and protected merge are required after the correction;
the failed product head does not inherit acceptance from another head.

The same change records this decision in the central record and paired #6840 and
#6880 issue comments. No registry or complete native product credit is claimed.
