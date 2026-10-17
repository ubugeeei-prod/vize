# Align documentation helpers with their runtime types

Refs [#6830](https://github.com/ubugeeei-prod/vize/issues/6830),
[#9200](https://github.com/ubugeeei-prod/vize/pull/9200).

Full Check [38988499789](https://github.com/ubugeeei-prod/vize/actions/runs/38988499789)
at `e2b883eed6378eb7ada73bf0220d89a185a3d6fd` stopped the JS/TS job
before native construction: the documentation project did not declare the
ES2024 promise API used by its preview waiter, and its outline helper retained
a nullable href after an optional-chain assertion.

Declare ES2024 library types for these Node-hosted helpers, preserving their
existing no-emit target and strict checks. Assert that the outline href is a
string before checking its hash prefix, so the exact same real navigation
continues with a non-null string. No preview resource, pointer, timeout or
assertion coverage is removed.

The repository's native TypeScript checker passes `docs/tsconfig.json` after
these changes; the two helpers and project configuration pass formatting and
lint. The failed hosted run remains failed; the full JS/TS task still needs
fresh Actions execution after this repair.
