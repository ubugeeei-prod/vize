# Complete original props-style CLI contract (#7988)

PR #8150 actually merged as signed commit
`eed471b4424b922b878bc35cac765dc7274e5736`. Its protected Check run
37639150378 passed 28 service-mode cases, typed-option validation/layer merging
and the reactive-defaults companion. The default is `only-when-assigned`;
explicit `never` preserves the old props-object preference. No production
behavior change is needed for this follow-up.

The original closure audit found that those service cases simplified the
reported union type and template and did not execute the original CLI command.
The new corpus preserves the complete `MyBadgeA.vue`, `MyBadgeB.vue`, reported
three-rule configuration and `lint -f plain --help-level short` arguments.
Twelve cases exercise default and all three modes, empty options, explicit off,
severity, matching-entry overrides and the compatible props-object preference
with the companion rule disabled. A separate bare-macro input distinguishes
`always` from `only-when-assigned` and `never` through the actual CLI.

Both complete plain bytes and complete JSON vectors are authored and checked
against the source-built `CARGO_BIN_EXE_vize`. Two fresh executions per format
retain all original source/config bytes and identical output. Invalid modes
and unknown fields must fail closed before linting. A source-custody guard pins
all 42 fixture files and checks that original inputs/config still match the
preserved issue verbatim.

Fresh exact-source Actions and protected queue acceptance, actual merge and
original issue completion remain pending. The current NAPI surface does not
expose this configurable mode; it is a separate unfinished public surface,
not an added gate for the original CLI question. Installed-release replay is
tracked by the release lane. No native linter replacement or broader parity
claim follows from this CLI corpus.
