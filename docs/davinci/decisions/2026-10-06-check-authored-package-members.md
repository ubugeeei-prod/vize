# Physical package report membership

Decision: [#8099](https://github.com/ubugeeei-prod/vize/issues/8099#issuecomment-6010758592).

The existing real-JavaScript Nuxt/Vite workspace cell at source `f0a64efd`
reported nonexistent `packages/ui/BadgeCard.vue` and `packages/ui/Missing.vue`
in complete CLI JSON. The actual component is `packages/ui/src/BadgeCard.vue`.
The missing-module vector still contains the genuine TS2307 diagnostic.

The package resolver intentionally preserves missing Vue candidates with their
native probe paths. These records permit native module resolution and retain
invalidation authority; removing them would change more than public membership.
The CLI import walk instead admits those records unconditionally into its
authored reported-file set. Require a physical file at that admission boundary
in both existing package-source branches. Preserve routes, mirror registrations,
traversal, native errors and every underlying resolver byte.

The differential corpus copies six entire authored workspace inputs, with
byte/hash pins and origin. The existing import-discovery Rust test compares
the complete authored source set for the original import and missing-module
edit. It separately requires the synthetic candidate/native probe route to
remain present and compares all source inputs after collection. Current macOS
canonical roots are used for expected filesystem identity.

This is a separate small CLI fix associated with #8099. The existing five-tool
coverage PR keeps whole stock/Vize CLI diagnostics, public metadata, both editor
profiles and framework runtime assertions. Current stock Nuxt 3 Node/config
errors and the Vite referenced-workspace editor TS7016 finding remain unresolved
and must not be accepted as clean results or hidden by this change.

No extra pipeline, serialization, provider API, fallback, native process or
benchmark budget is introduced. Source Actions, protected full suites, actual
merge and released consumer proof remain pending.
