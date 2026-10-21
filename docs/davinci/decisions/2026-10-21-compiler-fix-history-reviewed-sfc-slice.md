# Compiler fix-history reviewed SFC slice

Tracks [#6880](https://github.com/ubugeeei-prod/vize/issues/6880).

## First bounded review slice

The compiler history index now distinguishes a fixture link from a reviewed
semantic scope. Five existing SFC cases are the first reviewed slice:

| Fix           | Differential case                             | Reviewed behavior                                                                                   |
| ------------- | --------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `ceb0fda9c18` | `compiler/sfc/imported-component-before-prop` | A component import wins over a same-name prop in separate-template output.                          |
| `e9586daff99` | `compiler/sfc/scoped-css-v-bind`              | CSS v-bind names agree between populated CSS and useCssVars output.                                 |
| `b01f36701cd` | `compiler/sfc/dynamic-loop-ref-for`           | A dynamic ref inside v-for emits ref_for in separate-template output.                               |
| `ff008f61e83` | `compiler/sfc/computed-component-unref`       | Computed component tags are unwrapped across ordinary, conditional, loop, cached, and dotted paths. |
| `466fa309502` | `compiler/sfc/custom-directive-child-patch`   | A custom directive on an element with children retains NEED_PATCH.                                  |

Each case preserves its authored input and effective options plus all seven
public SFC Result facets: code, CSS, map, errors, warnings, bindings and macro
artifacts. The differential manifest pins every input, option, reference and
historical observation by SHA-256. The history generator now verifies those
digests, the authored witness symbol, the fix provenance, the complete-result
contract and the explicit fixture link before it accepts a scope review.

The machine-readable review ledger records both the requirements covered and
the remaining scope for every case. Regenerate and verify the index with:

```sh
vp node tools/commands/fixtures/compiler-fix-history.ts --write
vp exec oxfmt docs/davinci/plan/compiler-fix-history-index.json docs/davinci/plan/compiler-fix-history-reviews.json
vp node tools/commands/fixtures/compiler-fix-history.ts --check
```

## Limits

This slice reviews five bounded SFC output scopes. It does not mark any fix
subject complete. All 626 candidate subjects retain whole-fix target and
dialect review pending, the semantic denominator remains unset, and native
acceptance remains zero. The other 25 fixture-linked subjects still await even
their first bounded semantic review. Exact-head Actions validate the committed
generator and current product fixtures; historical captures alone do not do so.
