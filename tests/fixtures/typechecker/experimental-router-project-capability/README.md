# Project-owned experimental route props capability

This is a source payload and mutation-plan fixture for issue #10579. It grants
no production, protocol or mutation qualification credit yet.

`artifacts.json` binds unchanged package, public-entry and shared-declaration
members of the actual published `vue-router@5.4.0` archive and the independently
qualified source build at upstream commit
`593d7ad9ff99e95c24be28e962717df0fb2be763`. Both manifests report `5.4.0`.
Only the source-built provider actually exports `_RoutePropsForFilePath` from
`vue-router/experimental`. The source payloads are never injected as replacement
provider declarations into a consumer.

`mutations.json` declares future actual-provider controls and their expected
admission classes. Complete real SDK export/program packets must be captured
and frozen independently before these rows receive qualification credit. Tests
must mutate the real installed artifact or re-export its existing declarations;
they must not synthesize a helper. Candidate admission output is not an oracle.

The initial source graph tests read these real payloads, preserve raw package
condition order and follow actual type exports. They test simultaneous providers,
missing declarations, real star re-exports, source shadowing and warm graph reads.
They do not prove TypeScript host invalidation or installed-editor file watching.

`profile-schema.json` requires actual operation-boundary read accounting. Disabled
admission reads no provider files. Warm transforms and queries do not poll
provider-only declaration inputs. Shared config/provider paths retain their
original compiler observations and report their reads separately. Mutable
admission requires an actual client watch ACK and publication generation checks;
unknown or unsupported watch coverage is an explicit refusal.
