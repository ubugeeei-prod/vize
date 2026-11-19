# Handoff: refactor/glyph-f4-tags-on-l1-wip (#6847 F4 family 1)

Delete this file before merge. Paused 2026-11-19 for the machine switch:
**nothing was built or measured.** The disk stayed at 58 GB free, below the
60 GB build threshold. The code below has never been compiled.

## What is on the branch (WIP, uncompiled)

- `davinci/vize_l1/src/markup/htmlish/lex/pull.rs` (new): `Lexer::pull_tag(start)
  -> PulledTag { NotTag | Open{end,terminated} | Close{end,terminated} }` and
  `Lexer::sink_mut`. Both are generic, so they are instantiated only in Glyph.
  That keeps the L1 codegen-unit partition unchanged, which was the lesson
  from #13407. The method runs the shared state machine from `<` only through
  the tag states and returns once the state goes back to `Text`/`InRCDATA`,
  so the caller keeps owning text, comments, raw text and interpolation.
  `<!`, `<?` and a non-tag first byte return `NotTag` with no events. At EOF
  it calls the existing `recover_incomplete_tag_at_eof`, which is now
  `pub(super)` (a visibility change only). It is exported as
  `vize_l1::markup::htmlish::lex::PulledTag`.
- `crates/vize_glyph/src/template/formatter/tag_lex.rs` (new) has two sinks:
  - `TagSink` builds `ParsedAttribute`s from events. The raw name is the first
    `on_attrib_name`/`on_dir_name` start up to `on_attrib_name_end`. The value
    is the first data or entity start up to the `on_attrib_end` index, `""`
    when there is no data, and `None` for `NoValue`.
    `on_missing_attribute_value_at_eof` is mapped to `Some("")`, as the
    scanner does. Raw names and values are passed as `&str` slices, which
    removes one `CompactString` per name; this is the expected perf headroom.
  - `CloseSink` rejects `</ div>` (name not at `start+2`), as the scanner
    does.
- **Experiment switch, to be removed before any PR:** `tag_lex::L1_TAGS`,
  `experiment_set_l1_tags` (lib.rs, template.rs), the `opening`/`closing`/
  `immediate_empty` dispatchers, and `crates/vize_glyph/tests/zz_l1_tag_experiment.rs`.
  The test reads the corpus list from `GLYPH_CORPUS` and compares the scanner
  and L1 routes in one binary over four option sets × {LF, CR} × {SFC, raw
  template}. `format_inner`, `child_width.rs` and `preserved_text.rs` call
  sites already go through the dispatchers. The old `tags.rs`/`parse_closing_tag`
  are still present for the comparison.
- The corpus list script is in scratchpad `f4/corpus.sh` (6,781 files):
  formatter fixtures, every `.vue` file, glyph/l1/armature `.rs`/`.snap`, and
  tests html/json. Feeding non-Vue files as templates stresses the `<` edge
  cases.

## Scanner vs L1 lexer: differences found by reading the code

| # | input | scanner | L1 lexer |
|---|-------|---------|----------|
| A1 | `<1x>`, `<_x>`, `<-x>`, `<:x>`, `<.x>` | tag (name chars `[A-Za-z0-9_:.-]`, any first) | text (first byte must be `[A-Za-z]`) |
| A2 | `<div$x>`, `<a日>`, `<a"b>` | name `div`, rest becomes an attribute | name `div$x` (name runs to ws/`/`/`>`) |
| B | attr name with a quote, `a"b="c"` | name includes the quotes | `InAttrName` splits: error `UnexpectedCharacterInAttributeName`, value starts |
| C | unquoted value containing `/`: `href=/x`, `src=a/b.png` | value stops at `/`, `/` then sets self-closing: semantics-changing output | `/` is a value byte (HTML spec) |
| D | `/` mid-tag: `<div / class="a">` | `self_closing = true`, keeps going | error `UnexpectedSolidusInTag`, not self-closing |
| E | form feed `\x0c` in a tag | an ordinary name byte | whitespace |
| F1 | `</ div>` | text | close `div` (handled: rejected in `CloseSink`) |
| F2 | `</div$x>`, `</div/>`, `</a"b>` | name `div`/`a`, skips to `>` | name `div$x`/`div/`/`a"b` (closing name ends only at ws/`>`) |
| F3 | `</日本>` | text | close with `InvalidFirstCharacterOfTagName` |
| G | `v-bind[a b]`, `:foo[a b]` (dynamic-looking bracket with ws/`=`/`>`/`/` inside) | not dynamic, so the name ends at the space | `InDirName`/`InDirArg` `[` opens a dynamic arg, so the name includes `[a b]` |
| H | `a=` at EOF | `Some("")` | `NoValue` (handled by overriding `on_missing_attribute_value_at_eof`) |
| I | `&` in values (`a && b`) | plain byte | `InEntity` + `try_decode_entity` attempt, then rewind: **perf cost on every `&&`** |

They agree on: whitespace around `=`, quoted values containing `>`,
`a="b"c` adjacency, an unclosed quoted value at EOF (value to EOF, tag
inferred), `<` inside a name, dynamic args `:[`/`@[`/`#[`/`.[`/`v-bind:[`
(both use `dynamic_argument_boundary`), the RCDATA names (the pull stops at
`>` before RCDATA), and EOF inside a tag. All of this still needs the
experiment to confirm.

In every divergent row the L1 reading is the compiler's reading. The
scanner's reading changes semantics in A1, A2, C, D and F2 (it reprints a
different tag or attribute set than the compiler sees). Byte-identity is
required on the corpora. On non-corpus edge cases the brief says to keep the
scanner's behaviour. For A2/F2 that means truncating L1's name at the first
non-scanner byte, which re-scans bytes. **Decision for the maintainer:** take
the L1 reading for rows A–G as a listed fix (pinned fixtures + ledger rows),
or keep the scanner's behaviour.

## Next steps

1. When `/bin/df -g /Users` shows ≥ 60 GB free, run
   `nix develop --command bash -c 'GLYPH_CORPUS=<list> cargo test -p vize_glyph --test zz_l1_tag_experiment -- --nocapture'`.
   Fix compile errors first; none have been fixed yet.
2. Classify every DIFF line against the table, and add a refusal or mapping
   where the corpus hits a row. `TagSink.refused` is set for B and D but not
   yet acted on.
3. Pin each table row with a unit test that records base output.
4. Delete the scanner in the same PR: `tags.rs` parse fns, `helpers::parse_closing_tag`,
   `is_tag_name_char` and its test, `parse_immediate_empty_closing_tag`. Also
   delete the experiment switch and test. Change the call sites to `&str` tag
   names (no `String` per tag). Replace `child_width.rs`'s `parse_closing_tag`
   lookahead with a one-slot `(pos -> name, end)` cache that the main loop
   reuses, so no closing tag is lexed twice. `hugged_closing_name` still
   compares closing bytes by hand; it belongs to this family or the next.
5. Perf: `parse_opening_tag` is about 14.8k exclusive out of ≈111k inclusive
   on `formatter_template_complex` (244k). The per-byte stop check in
   `pull_tag` (`matches!(state, Text|InRCDATA)`) could become a check on the
   `>` byte. Watch row I (`&&` entity attempts); if it shows in the hotspots,
   add an L1 profile const that skips entity decoding in attribute values. The
   pull path would fold it at compile time.
6. Proof (unchanged from the brief): `parity.sh` with base and head
   `--profile ci` binaries (PARITY over about 11k files); `build-receipt.ts`
   and `differential-formatter-cli.test.mjs`; `cargo test -p vize_glyph -p vize_l1`;
   the release-profile `shipped-profile-measure.yml` table. That workflow was
   still not on main at pause time: check
   `git ls-tree origin/main .github/workflows/shipped-profile-measure.yml`.
   Also: ci-opt `level-instruction-counts` interleaved twins, a local fat-LTO
   size probe under the fleet lock, PR perf observation, and the
   skipped-on-PR gates (fmt, clippy `-D warnings -D clippy::wildcard_imports`,
   assertion-lint, source lengths, storage policy).
