Closes #13295

Depends on #13288; its two commits sit at the bottom of this branch and drop out on rebase once it merges.

**What was wrong.** Pug entity-escapes `"`, `&`, `<` and `>` in attribute values. Its source map was exact only for values that render unchanged; any other value mapped as one synthesized block over the whole attribute. So arrow handlers, `&&` and comparisons lost their columns. Under the content mapper, errors after the escape were dropped or reported at the value start. `vize check` (native), which maps through the derived HTML with its `&lt;` entities, reported them at several wrong columns, for example 3 rows for one error.

**Fix** (`davinci/vize_l1_to_l2/src/lower/pug`):
- A literal attribute value now maps per char. Unchanged runs stay verbatim, and each escaped or JS-cooked char maps to the bytes that spell it. `cook_body` reports the raw range of every cooked char, and `string_literal` is built on it, so evaluation is unchanged.
- The authored-order view, read by canon, lint and the IDE, keeps raw `<`, `>` and any `&` that cannot start a character reference. Vue decodes both spellings to the same value, so the derived text keeps the expression's columns. `"` and `&` before an alphanumeric or `#` stay escaped.
- The compiled `PugRendering::Pug` output is byte-identical to before.

**Tests.**
- `lower::pug::escaped_value_tests`: exact derived HTML and per-span maps for both renderings. Covers comparisons, `&&`, arrows, `&quot;`, `&amp;` before a reference start, JS `\'` escapes, unescaped `!=` values, and computed values (which still map to the whole attribute).
- `content_mapper_tsgo_cli::pug_escaped_attribute_values_13295`: 16 HTML/Pug pairs under tsc 7.1.0-beta, including the issue repro, `>=`, block handlers, both quote styles, `#id.class` heads, `v-for`, multi-line attribute lists and `#{}` text. Each Pug file reports what its HTML twin reports, once, with every member error on the authored Pug column, and `vize check` matches the mapper row for row.
- Davinci pug corpus, `vize_l1_to_l2` (953 tests), and the patina, maestro and canon pug suites pass. (Two canon fix-history tests need an installed Vue package and fail the same way on main.)

**Real projects** (dho-web-client, 211 Pug SFCs, baseline main vs this branch):
- `vize check`: byte-identical output.
- `vize lint`: the same 9,077 messages by file, rule, message, line and severity. 798 move, each narrowed inside its old range, for example `no-undefined-refs` now on the name instead of the whole `@click` value.
- The pinned primevue has no Pug templates.

**Known limits** (unchanged behaviour, each still reported once):
- A JS escape inside a single-quoted value (`\'`) still offsets later columns: the derived `'` is one byte where the authored `\'` is two.
- `&attributes(...)` renders through the object format, whose values carry no authored spelling.
- An HTML template that spells `&lt;`/`&amp;` entities inside a bound value has the same native mapping gap. That is a separate canon issue.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
