# Art editor lint parity

Issue: [#7945](https://github.com/ubugeeei-prod/vize/issues/7945).

Both editor diagnostic entry points run the project-configured Patina SFC
linter for `.art.vue`, in addition to Musea-specific lint. Patina already owns
the original-source `<variant>` template ranges; the LSP uses that same result
and the existing UTF-16 conversion rather than a second variant parser.
Musea findings retain their existing dedicated collector, source, severity
and documentation links; the shared result excludes those findings to avoid
publishing them twice when Musea rules are explicitly configured.

The authored corpus preserves the issue's anchor and image findings. Tests
cover both `art-vue` and `vue` language IDs, full and lint-only diagnostics,
CLI parity, original variant positions, an astral-character offset control,
and lint-disabled behavior. Existing project configuration applies through
the shared Patina constructor. This legacy regression adds no Davinci native
acceptance credit; Art typechecking remains separately tracked.

The paired original 400-provider qualification includes the exact Art lint test,
Art collector helper and diagnostic-service source paths changed by this slice. The
original corpus, 79 response answers, 534 acknowledgements, notification
controls and immutable source/input custody remain unchanged. This admits the
actual correctness change to the existing full comparison; it supplies no
performance credit until the exact-source original gate actually executes.
