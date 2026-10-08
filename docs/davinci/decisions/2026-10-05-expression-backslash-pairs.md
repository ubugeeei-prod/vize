# Code-position backslash pairs in the expression guard (#7808)

The scheduled Fuzz run [37195977813](https://github.com/ubugeeei-prod/vize/actions/runs/37195977813)
failed `js_ts_expression` on source `d544ed9a7d696d07a3f753df7c9e5edea2257d2f`.
Its authenticated reproducer is committed without transformation as
`tests/fuzz/regressions/js_ts_expression/issue-7808.input`: 22,933 bytes,
SHA256 `8670c91bace6df50473b3ad261b306446fb59b345d8d644a9771aa5aa4a0e5cb`.
All 79 NUL bytes and 3,012 CR bytes are retained. Artifact `11301588201` has
ZIP SHA256 `39a0c4ceeab12335565cee40c34e7e9c8c3c92d8010003e8751cfbbeb0a82331`.

The shared guard consumed code-position backslashes individually. The pinned
OXC lexer's `identifier_unicode_escape_sequence` consumes the next character
of an invalid escape, including a second backslash. Therefore an even run leaves
the next quote/backtick/slash live, while an odd tail consumes that character.
Sixteen backslashes at offsets 11–26 of the original input leave backtick 27
as a real template opener. The old guard instead neutralized it, then opened a
phantom template at closing backtick 47, hiding the following recursive brackets.
The original sanitizer log reports stack overflow. No general lexer equivalence
or universal scanner correctness is inferred from this boundary repair.

The existing invalid-escape arm now consumes a following backslash alongside
its existing quote/backtick/slash cases. Expression depth remains 31, with the
same speculative-angle, operator and numeric budgets. The parser, defaults,
pipeline stages, public APIs and normal dependency graph are unchanged.

The production legacy integration test uses the exact corpus input. It requires
rejection before parsing, complete unchanged output from the two expression
rewrites and empty slot extraction. Odd/even controls exercise real template,
string and comment boundaries; ordinary Unicode/literal escapes remain accepted.
The corpus seeder now includes committed expression regressions directly, so
`Fuzz mode=replay` executes these exact bytes rather than depending on a grown
cache. Its tooling test verifies byte/hash custody and repeated corpus regeneration.

Local isolated seeder regression passed. The attempted standalone guard build
could not resolve its OXC identifier dependency and is not execution evidence.
Source-built integration Actions and exact-source sanitizer replay are required
before admission; the full differential suites and unchanged instruction caps
run in the protected merge queue. Actual merge, issue closure and the supported
release refresh remain pending. This fixes a shared production guard and grants
no product-default replacement or completed Davinci milestone.

The issue author is the authenticated GitHub Actions bot, ID 41898282. The
meaningful implementation commit retains its verified GitHub noreply Co-author.
