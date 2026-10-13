# Reactive props rest binding (#8835)

The report on v0.443.0 contains a complete `Child.vue` whose template spreads
`rest` from `const { label, ...rest } = defineProps<...>()`. Vize removed the
declaration without generating a rest binding; references in setup were also
rewritten to all props, incorrectly retaining excluded keys.

Both script emission modes now import Vue's existing `createPropsRestProxy`
and declare the original rest variable inside setup. Original source keys are
excluded in declaration order, including aliases and quoted keys. References
to the rest binding retain that binding, while ordinary destructured properties
continue using the existing AST transform and lexical shadow handling.
The rest binding also overrides Croquis's prop classification so client and
SSR templates reference setup state. Quoted runtime defaults use the existing
property-key serializer.

The generated declaration ends with a semicolon. Vize emits it ahead of the
remaining setup statements, so an unterminated declaration would let a
following statement that starts with `(` or `[` continue the initializer. The
`FollowedByCall` control retains that case.

The reporter's complete Child SFC and alias/default/rest-only controls live in
`tests/_fixtures/differential/compiler/props-rest-runtime-8835/`. Rust tests
compile the report in both script modes and SSR modes. The runtime regression
executes actual compiled modules with Vue's real helper and observes exclusion
and changing getter values; it requires the source-built native binding.

Actions must pass on the published PR head before queue entry. Publication and
fresh installed-consumer acceptance remain required before closing #8835.
