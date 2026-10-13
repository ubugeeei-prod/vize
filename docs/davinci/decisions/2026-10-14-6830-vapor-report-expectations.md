# Keep the Vapor report cases without pinning the expected snapshot

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830). Reports:
[#8624](https://github.com/ubugeeei-prod/vize/issues/8624)–[#8627](https://github.com/ubugeeei-prod/vize/issues/8627).

`vapor-third-party-reports.test.mjs` hashed the whole live
`tests/expected/vapor/component.snap` and checked the test runner's source
for literal strings, including the exact coverage floors. Regenerating the
Vapor expectations for a newer `@vue/compiler-vapor` release, or raising a
coverage floor, failed the test even though the reported cases were kept.

The test now checks what the reports need: the recorded successor, wrapper
controls and public emissions under `tests/_fixtures/` keep their hashes; the
reported KeepAlive case is still present in the live expectations with its
original input; the coverage floors are at least 117 Vapor and 743 total
passes; and the runner keeps no known failures.
