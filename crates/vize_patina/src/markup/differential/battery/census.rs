//! Exact committed battery census and its append-only change accounting.

use super::super::JsxComparison;
use super::BatteryCensus;

/// The committed census. Re-pinned deliberately, in both lanes at once.
pub const PINNED_BATTERY_CENSUS: BatteryCensus = BatteryCensus {
    templates: 37,
    rule_fixtures: 869 + 38 + 1 + 1 + 5 + 1 - 5
        + 2
        + 1
        + 1
        + 2
        + 7
        + 3
        + 15
        + 4
        + 2
        + 2
        + 6
        + 1
        + 5
        + 2
        + 5,
    // Original 1057/87 census plus non-literal calls in valid_v_slot (3 + 1),
    // require_toggle_inside_transition (3), the v-else-if/v-is/v-pre rules (38
    // templates over 42 calls, one `v-pre` helper, #8856) and v_bind_style (one
    // `.prop` template, one fix helper call). The kebab-case KeepAlive child
    // control adds one literal template (#9076). The v-on modifier regressions
    // add five literal templates: nativeish without a handler, custom.stop,
    // Custom, custom without a handler, and custom with a handler (#9203).
    // Their six additional dynamic-template calls retain the complete modifier
    // matrices in rule tests; the scanner deliberately records those as skips.
    // KeepAlive's complete source/repair packets add five nonliteral calls;
    // their explicit loops remain in rule tests, and all literal samples remain.
    // Empty binding values add one nonliteral call over seven real-parser
    // inputs; that loop and the complete CLI reports run in ordinary tests.
    // The `@vue:` vnode-hook exemption adds one literal template over two calls.
    // placeholder-label-option's rewritten unit tests (#9749) loop over
    // template lists: five fewer literal templates and one more nonliteral call.
    // The combobox exemption tests (#9750) add three nonliteral calls.
    // Dynamic slot v-for key ownership (#9924) adds two literal templates.
    // Patina rule guards (#9870, #9871) add one nonliteral call and skip.
    // Patina output policy coverage (#9869, #9874) adds one literal template.
    // Patina fix regressions (#9872/#9873) add five helper-driven calls whose
    // complete authored sources and edits are asserted by the focused tests.
    // HTML quote escape coverage (#6097) adds two focused calls; the source
    // variable remains an intentional scanner skip while the literal dedups.
    // The attribute-hyphenation range regression adds one nonliteral call.
    // Kebab-case and `is`-cast v-on listeners (#10822) add one literal template;
    // both are counted in the final `+ 2`.
    // Dynamic-role regression matrices (#10926) add two nonliteral calls;
    // all eleven inputs execute in focused tests while literal coverage stays exact.
    // The pinned 37-case / 148-whole-packet listbox controller (#10925) adds
    // one dynamic legacy.lint_template(&source) call and one scanner skip.
    // All 913 existing literal fixtures remain; native/legacy host tests and
    // 298 source CLI processes independently retain the complete new packets.
    // Native input-label history adds seven nonliteral legacy calls: three
    // original/pair/requery calls and two each for duplicate/select controls.
    // Their complete outputs run in the focused native tests; each remains
    // honestly counted as a variable-input skip in this literal-only scan.
    // Native placeholder-option history adds three nonliteral legacy calls:
    // the original snapshot query, complete pair baseline and pair requery.
    // All four whole originals and the policy matrix run in focused tests;
    // their variables remain skips without changing the 913 literal fixtures.
    // Native static-is pairs, disabled controls and refused owners add three
    // nonliteral legacy calls. Their complete Case/Observation tests execute;
    // this literal-only scanner records the three calls as skips, not fixtures.
    // Literal attribute ignoreTags adds eight focused lint_template calls:
    // seven retain source variables and remain counted scanner skips. The
    // unselected-rule control adds one new literal, <MyWidget fooBar="x" />,
    // whose complete Relief/L2 facade comparison contributes four trace lines.
    // The nineteen original attribute tests only move to their owning leaf;
    // their unchanged calls and literal sources remain in the same scan.
    // Native numeric modifier pairs, applied fixes and refusal controls add
    // five nonliteral legacy calls; full source-unit observations execute
    // without adding a literal fixture or original history coordinate.
    // Deprecated-native bare observations, numeric fix requery and disabled
    // controls add three nonliteral legacy calls. All originals stay intact;
    // the literal-only scan counts these calls as skips, not new fixtures.
    // Valid-v-on's whole legacy/native observer adds one nonliteral call;
    // absent-handler unit controls add no literal history fixture or trace.
    // Current native callback controls add eight calls: two literal inputs and
    // six variable-input observations. Every full facade comparison still runs;
    // these add thirteen retained trace lines, not semantic qualification credit.
    // Real-project parity regressions add seven literal fixtures over ten calls;
    // three helper-driven calls remain honest dynamic-input scanner skips.
    // Script-binding and precise-range regressions add three literal fixtures
    // over four calls; the helper-driven call remains an honest scanner skip.
    // Real-project lint parity adds fifteen literal fixtures over seventeen
    // calls; three helper-driven calls remain explicit scanner skips. The
    // receiver and expression-tokenizer regressions add two focused dynamic
    // calls, both retained as explicit scanner skips.
    // The final real-project parity pass adds four literal fixtures and calls;
    // their complete traces add thirty-four lines.
    // The same-element v-if/v-for parity probe adds one nonliteral call and
    // remains an explicit scanner skip while its focused rule test executes.
    // XML-name and Vue 2 slot-scope parity add two literal fixtures and calls;
    // their complete traces add ten lines.
    // Slot outlets and ref reassignment add two literal fixtures and calls;
    // their complete traces add thirteen lines.
    // Scriptless custom-event casing and the real-project accuracy regressions
    // add six literal fixtures, nine calls, three skipped projections, and
    // fifty-one complete trace lines.
    // Generated v-for parser recovery adds one literal fixture over two calls;
    // the source variable is one explicit scanner skip and adds twelve lines.
    // Real-project spread, dynamic-component and foster-parenting regressions
    // add three literal fixtures over four calls; the loop input stays a skip.
    // Recovered-table component evidence and inherited slot iterations add two
    // more literal fixtures and calls over nineteen complete trace lines.
    // Keyed nested-loop and keyed-root slot fallbacks add two literal fixtures
    // and calls over twenty-seven complete trace lines.
    // Preserving authored table-section cells removes the four trace lines that
    // belonged only to the parser-inserted row and keeps every authored node.
    // Real-project configurable allowances add five literal fixtures over
    // eight calls. Three loop-driven observations remain explicit scanner
    // skips, and the retained facade traces add twenty-nine complete lines.
    rule_fixture_calls: 1057 + 4 + 3 + 43 + 2 + 1 + 11 + 5 + 1 + 2 - 5
        + 3
        + 2
        + 1
        + 1
        + 5
        + 2
        + 2
        + 2
        + 1
        + 7
        + 3
        + 3
        + 8
        + 5
        + 3
        + 1
        + 8
        + 10
        + 4
        + 17
        + 2
        + 4
        + 1
        + 2
        + 2
        + 9
        + 2
        + 6
        + 2
        + 8,
    rule_fixture_skipped: 87
        + 4
        + 3
        + 1
        + 1
        + 6
        + 5
        + 1
        + 1
        + 3
        + 1
        + 5
        + 1
        + 1
        + 2
        + 1
        + 7
        + 3
        + 3
        + 7
        + 5
        + 3
        + 1
        + 6
        + 3
        + 1
        + 3
        + 2
        + 1
        + 3
        + 1
        + 1
        + 3,
    matrix: 90,
    template_lines: 6316 + 233 + 5 + 10 + 25 + 7 - 49
        + 31
        + 42
        + 4
        + 13
        + 51
        + 18
        + 102
        + 34
        + 10
        + 13
        + 51
        + 12
        + 51
        + 27
        + 29
        - 4,
    restructured: 0,
    jsx: JsxComparison {
        roots: 14,
        refused: 1,
        lines: 118,
    },
};
