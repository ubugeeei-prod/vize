## Summary

`vapor/prefer-static-class` maps to `vue/prefer-separate-static-class`, but it reported a different range: the whole `:class` binding for a constant. It also cut static fragments out of a template quasi such as `callout__${type}`. In the Real Project Matrix (#12336) the mismatch produced about 300 false positives and 150 false negatives.

The rule now reports each static class node the way eslint-plugin-vue does:

- a string literal;
- a whole template quasi bounded by whitespace or the literal's ends, including its `` ` ``, `${` and `}` delimiters as ESTree ranges it;
- an array item;
- an object key whose value is `true`.

A quasi that is part of a dynamic class, such as `badge--` in `` `badge--${variant}` ``, is skipped whole.

The autofix still replaces the whole binding.

## Tests

- New rule test covering a literal, a whole quasi, a fragment quasi and array and object items.
- The static-class cohort (`crates/vize_patina/tests/fixtures/...`) re-pins each diagnostic range to the literal; edits and applied bytes are unchanged. Re-pinned to match: the cohort `cases.json` sha, the oracle-index snapshot shas, the manifest oracle sha and `differential-linter-sfc-contract` pins, the battery census (+1 call, +1 skip, with accounting), the snapshots and the docs anchors.
- Differential case `linter/prefer-static-class-nodes-12336`.

No Vapor-only reporting case is kept. The rule reports exactly the nodes upstream reports, and only its autofix, which moves the class into a static attribute, stays Vapor-specific.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
