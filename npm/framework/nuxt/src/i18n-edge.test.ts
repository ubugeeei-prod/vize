import assert from "node:assert/strict";
import { test } from "node:test";

import { injectNuxtI18nHelpers } from "./i18n.ts";

// No i18n usage inside setup() => code returned completely unchanged.
void test("leaves code untouched when no runtime i18n helper is used", () => {
  const input = `
export default {
  setup(__props) {
    const x = 1;
  }
}
`;
  assert.equal(injectNuxtI18nHelpers(input), input);
});

// No setup(__props) at all => code returned unchanged.
void test("leaves code untouched when there is no setup(__props)", () => {
  const input = `export default { data() { return {}; } }`;
  assert.equal(injectNuxtI18nHelpers(input), input);
});

// Every helper maps to its destructure specifier exactly once, in source order.
void test("injects all six helpers with the correct specifiers exactly once", () => {
  const out = injectNuxtI18nHelpers(`
export default {
  setup(__props) {
    $t("a"); $rt("b"); $d(new Date()); $n(1); $tm("c"); $te("d");
  }
}
`);

  assert.match(
    out,
    /const \{ t: \$t, rt: \$rt, d: \$d, n: \$n, tm: \$tm, te: \$te \} = useI18n\(\);/,
    "all six helpers should be destructured with their canonical specifiers",
  );
  // Each specifier appears exactly once, and only a single useI18n() call is injected.
  assert.equal(out.match(/useI18n\(\)/g)?.length, 1);
  assert.equal(out.match(/t: \$t/g)?.length, 1);
  assert.equal(out.match(/rt: \$rt/g)?.length, 1);
  assert.equal(out.match(/te: \$te/g)?.length, 1);
  assert.equal(out.match(/tm: \$tm/g)?.length, 1);
});

// Repeated uses of the same helper still produce a single specifier.
void test("deduplicates repeated uses of the same helper", () => {
  const out = injectNuxtI18nHelpers(`
export default {
  setup(__props) {
    $t("a");
    $t("b");
    $t("c");
  }
}
`);
  assert.match(out, /const \{ t: \$t \} = useI18n\(\);/);
  assert.equal(out.match(/t: \$t/g)?.length, 1);
});

// _ctx.$t(...) and this.$t(...) are template/options globals (preceded by "."),
// excluded by the (?<![.\w]) lookbehind => no injection happens.
void test("does not trigger on _ctx.$t or this.$t member access", () => {
  const out = injectNuxtI18nHelpers(`
export default {
  setup(__props) {
    return (_ctx) => _ctx.$t("x") + this.$t("y");
  }
}
`);
  assert.equal(out.includes("useI18n()"), false);
  // Any "$t(" preceded by a word char (e.g. a custom member like foo.$t) is also ignored.
  const memberOut = injectNuxtI18nHelpers(`
export default {
  setup(__props) {
    foo.$t("x");
  }
}
`);
  assert.equal(memberOut.includes("useI18n()"), false);
});

// Existing destructure, then a NEW helper used AFTER it => merged into the existing
// destructure rather than emitting a second useI18n() call.
void test("merges a later helper into an existing useI18n destructure", () => {
  const out = injectNuxtI18nHelpers(`
export default {
  setup(__props) {
    const { t: $t } = useI18n();
    $d(new Date());
  }
}
`);
  assert.match(out, /const \{ t: \$t, d: \$d \} = useI18n\(\);/);
  assert.equal(out.match(/useI18n\(\)/g)?.length, 1, "no duplicate useI18n() should be emitted");
});

// Existing destructure but the helper use appears BEFORE it => a new const is injected
// at the top of setup, and the user-authored destructure below is left in place.
// Observed ordering side effect: this yields TWO useI18n() calls.
void test("injects above setup when a helper is used before the existing destructure", () => {
  const out = injectNuxtI18nHelpers(`
export default {
  setup(__props) {
    $d(new Date());
    const { t: $t } = useI18n();
  }
}
`);
  // Injected const sits right after the setup signature, before the first use.
  assert.match(out, /setup\(__props\) \{\nconst \{ d: \$d \} = useI18n\(\);\n/);
  // The pre-existing destructure is preserved untouched.
  assert.match(out, /const \{ t: \$t \} = useI18n\(\);/);
  // Observed behavior: the early-use branch does not merge, so two calls coexist.
  assert.equal(out.match(/useI18n\(\)/g)?.length, 2);
});

// SURPRISING (text-based regex): a "$t(" that lives only inside a string literal still
// matches and triggers injection, because the transform never parses JS.
void test("matches helper-looking text inside a string literal (no JS parsing)", () => {
  const out = injectNuxtI18nHelpers(`
export default {
  setup(__props) {
    const s = "call $t() here";
  }
}
`);
  assert.match(out, /const \{ t: \$t \} = useI18n\(\);/);
});

// SURPRISING (text-based regex): a "$d(" inside a line comment also triggers injection.
void test("matches helper-looking text inside a comment (no JS parsing)", () => {
  const out = injectNuxtI18nHelpers(`
export default {
  setup(__props) {
    // use $d() somewhere
    const x = 1;
  }
}
`);
  assert.match(out, /const \{ d: \$d \} = useI18n\(\);/);
});
