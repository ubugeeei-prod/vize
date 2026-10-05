# CSS ID selector range corpus

Issue #7981's complete Vue and JSON fences are preserved byte-for-byte. The
complete FlatCard/NestedCard files from #7055 and Note.vue from #7171 are also
original inputs. The manifest explicitly labels every authored control,
records each source hash/byte count, and links complete expected API diagnostic
vectors and public JSON/plain output.

The original's two ranges select `#main-banner` at 12:1–12:13 and `#banner-text`
at 17:3–17:15. Other cases retain original token widths through CRLF, astral
text, multiple styles, escaped IDs, comments/strings/attributes, repeated IDs,
selector lists and existing nested/functional/deep scope. Disable comments
retain their existing policy at the corrected selector line.

`css_id_selector_ranges.rs` passes every whole SFC through the public linter
API and compares every diagnostic field, authored slice and complete reports.
`css-id-selector-ranges.test.mjs` requires a source-built CLI receipt and all
33 complete JSON/plain attempts using the original config. Actual complete
observations are retained under `target/differential/`; neither test can skip a
missing binary or failed case. Rust CI also retains the complete API rows in
its existing nextest pr/full shard result artifact. Complete expectations also
retain the existing CSS rule help in the API and JSON/plain reports, including
with `--help-level none`; the direct CSS adapter preserves that established
behavior. This does not confer native-stage eligibility or permit rewriting
historical oracles.
