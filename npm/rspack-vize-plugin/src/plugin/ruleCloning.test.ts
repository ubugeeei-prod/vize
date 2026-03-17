import { describe, test } from "node:test";
import assert from "node:assert/strict";
import "./../test/setup.ts";
import { applyRuleCloning } from "./ruleCloning.js";

void describe("applyRuleCloning", () => {
  void test("does nothing when no vize loader is found", () => {
    const rules = [{ test: /\.css$/, use: ["style-loader", "css-loader"] }];
    const result = applyRuleCloning(rules as never, true);

    assert.equal(result.applied, false);
    assert.equal(result.clonedCount, 0);
  });

  void test("does nothing when .vue rule already has oneOf", () => {
    const rules = [
      {
        test: /\.vue$/,
        oneOf: [
          { resourceQuery: /type=style/, use: ["some-loader"] },
          { use: [{ loader: "@vizejs/rspack-plugin/loader" }] },
        ],
      },
    ];
    const result = applyRuleCloning(rules as never, true);

    assert.equal(result.applied, false);
  });

  void test("clones CSS rule and builds oneOf (nativeCss)", () => {
    const rules = [
      { test: /\.css$/, use: ["style-loader", "css-loader"] },
      {
        test: /\.vue$/,
        use: [{ loader: "@vizejs/rspack-plugin/loader" }],
      },
    ];

    const result = applyRuleCloning(rules as never, true);

    assert.equal(result.applied, true);
    assert.ok(result.clonedCount >= 1);

    // The .vue rule should now have oneOf
    const vueRule = rules[1] as Record<string, unknown>;
    assert.ok(Array.isArray(vueRule.oneOf));

    const oneOf = vueRule.oneOf as Array<Record<string, unknown>>;

    // Last entry should be the main loader fallback
    const mainBranch = oneOf[oneOf.length - 1];
    const use = mainBranch.use as Array<Record<string, unknown>>;
    assert.equal(use[0].loader, "@vizejs/rspack-plugin/loader");

    // CSS rule should now have vue exclusion
    const cssRule = rules[0] as Record<string, unknown>;
    assert.ok(cssRule.resourceQuery);
  });

  void test("clones SCSS rule alongside CSS (nativeCss)", () => {
    const rules = [
      { test: /\.css$/, use: ["style-loader", "css-loader"] },
      { test: /\.scss$/, use: ["style-loader", "css-loader", "sass-loader"] },
      {
        test: /\.vue$/,
        use: [{ loader: "@vizejs/rspack-plugin/loader" }],
      },
    ];

    const result = applyRuleCloning(rules as never, true);

    assert.equal(result.applied, true);

    const vueRule = rules[2] as Record<string, unknown>;
    const oneOf = vueRule.oneOf as Array<Record<string, unknown>>;

    // Should have cloned CSS + SCSS + possibly fallback + main loader
    assert.ok(oneOf.length >= 3);

    // Find the SCSS cloned rule
    const scssClone = oneOf.find((r) => {
      const rq = r.resourceQuery as RegExp;
      return rq && rq.test("vue&type=style&index=0&lang=scss");
    });
    assert.ok(scssClone, "should have a cloned SCSS rule");

    // The SCSS clone should have scope-loader first, user loaders in the middle, style-loader last
    const scssUse = scssClone!.use as Array<string | Record<string, unknown>>;
    const firstLoader = scssUse[0] as Record<string, unknown>;
    assert.equal(firstLoader.loader, "@vizejs/rspack-plugin/scope-loader");
    const lastLoader = scssUse[scssUse.length - 1] as Record<string, unknown>;
    assert.equal(lastLoader.loader, "@vizejs/rspack-plugin/style-loader");

    // Both original rules should have vue exclusion
    assert.ok((rules[0] as Record<string, unknown>).resourceQuery);
    assert.ok((rules[1] as Record<string, unknown>).resourceQuery);
  });

  void test("generates css/auto type for native CSS cloned rules", () => {
    const rules = [
      { test: /\.scss$/, use: ["sass-loader"] },
      {
        test: /\.vue$/,
        use: [{ loader: "@vizejs/rspack-plugin/loader" }],
      },
    ];

    const result = applyRuleCloning(rules as never, true);
    assert.equal(result.applied, true);

    const vueRule = rules[1] as Record<string, unknown>;
    const oneOf = vueRule.oneOf as Array<Record<string, unknown>>;

    // Find a cloned style rule
    const styleRule = oneOf.find((r) => r.resourceQuery instanceof RegExp);
    assert.ok(styleRule);
    assert.equal(styleRule!.type, "css/auto");
  });

  void test("always adds a CSS fallback rule for plain <style>", () => {
    const rules = [
      { test: /\.scss$/, use: ["sass-loader"] },
      {
        test: /\.vue$/,
        use: [{ loader: "@vizejs/rspack-plugin/loader" }],
      },
    ];

    const result = applyRuleCloning(rules as never, true);
    assert.equal(result.applied, true);

    const vueRule = rules[1] as Record<string, unknown>;
    const oneOf = vueRule.oneOf as Array<Record<string, unknown>>;

    // Should have a fallback that matches lang=css
    const cssFallback = oneOf.find((r) => {
      const rq = r.resourceQuery as RegExp;
      return rq && rq.test("vue&type=style&index=0&lang=css");
    });
    assert.ok(cssFallback, "should have a CSS fallback rule");
  });

  void test("preserves original rule type in clone", () => {
    const rules = [
      { test: /\.css$/, type: "css/module", use: ["css-loader"] },
      {
        test: /\.vue$/,
        use: [{ loader: "@vizejs/rspack-plugin/loader" }],
      },
    ];

    const result = applyRuleCloning(rules as never, true);
    assert.equal(result.applied, true);

    const vueRule = rules[1] as Record<string, unknown>;
    const oneOf = vueRule.oneOf as Array<Record<string, unknown>>;
    const cssClone = oneOf.find((r) => {
      const rq = r.resourceQuery as RegExp;
      return rq && rq.test("vue&type=style&index=0&lang=css");
    });
    assert.ok(cssClone);
    assert.equal(cssClone!.type, "css/module");
  });

  void test("deep clones use entries to avoid mutating originals", () => {
    const originalOptions = { modules: true };
    const rules = [
      {
        test: /\.css$/,
        use: [{ loader: "css-loader", options: originalOptions }],
      },
      {
        test: /\.vue$/,
        use: [{ loader: "@vizejs/rspack-plugin/loader" }],
      },
    ];

    applyRuleCloning(rules as never, true);

    const vueRule = rules[1] as Record<string, unknown>;
    const oneOf = vueRule.oneOf as Array<Record<string, unknown>>;
    const cssClone = oneOf.find((r) => {
      const rq = r.resourceQuery as RegExp;
      return rq && rq.test("vue&type=style&index=0&lang=css");
    });
    const clonedUse = cssClone!.use as Array<Record<string, unknown>>;
    // index 0 is scope-loader, index 1 is the cloned user loader
    const clonedLoader = clonedUse[1] as Record<string, unknown>;

    // Mutating the clone should not affect the original
    (clonedLoader.options as Record<string, unknown>).modules = false;
    assert.equal(originalOptions.modules, true);
  });

  void test("handles resolved file paths for vize loader", () => {
    const rules = [
      { test: /\.css$/, use: ["css-loader"] },
      {
        test: /\.vue$/,
        use: [
          {
            loader: "/path/to/node_modules/@vizejs/rspack-vize-plugin/dist/loader/index.mjs",
          },
        ],
      },
    ];

    const result = applyRuleCloning(rules as never, true);
    assert.equal(result.applied, true);
  });

  void test("skips '...' entries in rules array", () => {
    const rules: unknown[] = [
      "...",
      { test: /\.css$/, use: ["css-loader"] },
      {
        test: /\.vue$/,
        use: [{ loader: "@vizejs/rspack-plugin/loader" }],
      },
    ];

    const result = applyRuleCloning(rules as never, true);
    assert.equal(result.applied, true);
  });

  void test("handles `loader` shorthand (no `use` array)", () => {
    const rules = [
      {
        test: /\.scss$/,
        type: "css/auto",
        use: [
          {
            loader: "sass-loader",
            options: { sassOptions: { quietDeps: true } },
          },
        ],
      },
      {
        test: /\.vue$/,
        loader: "@vizejs/rspack-plugin/loader",
      },
    ];

    const result = applyRuleCloning(rules as never, true);

    assert.equal(result.applied, true);
    assert.ok(result.clonedCount >= 1);

    // The .vue rule should now have oneOf
    const vueRule = rules[1] as Record<string, unknown>;
    assert.ok(Array.isArray(vueRule.oneOf));

    const oneOf = vueRule.oneOf as Array<Record<string, unknown>>;

    // Last entry (main loader fallback) should have the loader in its use array
    const mainBranch = oneOf[oneOf.length - 1];
    const use = mainBranch.use as Array<string | Record<string, unknown>>;
    assert.equal(use[0], "@vizejs/rspack-plugin/loader");

    // SCSS clone should exist
    const scssClone = oneOf.find((r) => {
      const rq = r.resourceQuery as RegExp;
      return rq && rq.test("vue&type=style&index=0&lang=scss");
    });
    assert.ok(scssClone, "should have a cloned SCSS rule");
  });

  void test("handles `loader` + `options` shorthand", () => {
    const rules = [
      { test: /\.css$/, use: ["css-loader"] },
      {
        test: /\.vue$/,
        loader: "@vizejs/rspack-plugin/loader",
        options: { sourceMap: true },
      },
    ];

    const result = applyRuleCloning(rules as never, true);
    assert.equal(result.applied, true);

    const vueRule = rules[1] as Record<string, unknown>;
    const oneOf = vueRule.oneOf as Array<Record<string, unknown>>;
    const mainBranch = oneOf[oneOf.length - 1];
    const use = mainBranch.use as Array<Record<string, unknown>>;

    // Should be an object entry with loader + options
    assert.equal(use[0].loader, "@vizejs/rspack-plugin/loader");
    assert.deepEqual(use[0].options, { sourceMap: true });
  });

  void test("injects scope-loader between user loaders and style-loader in all cloned rules", () => {
    const rules = [
      { test: /\.css$/, use: ["css-loader"] },
      { test: /\.scss$/, use: ["sass-loader"] },
      { test: /\.less$/, use: ["less-loader"] },
      {
        test: /\.vue$/,
        use: [{ loader: "@vizejs/rspack-plugin/loader" }],
      },
    ];

    const result = applyRuleCloning(rules as never, true);
    assert.equal(result.applied, true);

    const vueRule = rules[3] as Record<string, unknown>;
    const oneOf = vueRule.oneOf as Array<Record<string, unknown>>;

    // Check every style rule (everything except the last main-loader fallback)
    for (let i = 0; i < oneOf.length - 1; i++) {
      const branch = oneOf[i];
      const use = branch.use as Array<string | Record<string, unknown>>;

      // First loader should be scope-loader, last should be style-loader
      const firstLoader = use[0] as Record<string, unknown>;
      const lastLoader = use[use.length - 1] as Record<string, unknown>;

      assert.equal(
        firstLoader.loader,
        "@vizejs/rspack-plugin/scope-loader",
        `branch ${i}: first loader must be scope-loader`,
      );
      assert.equal(
        lastLoader.loader,
        "@vizejs/rspack-plugin/style-loader",
        `branch ${i}: last loader must be style-loader`,
      );
    }
  });

  void test("CSS fallback rule has both scope-loader and style-loader", () => {
    const rules = [
      { test: /\.scss$/, use: ["sass-loader"] },
      {
        test: /\.vue$/,
        use: [{ loader: "@vizejs/rspack-plugin/loader" }],
      },
    ];

    const result = applyRuleCloning(rules as never, true);
    assert.equal(result.applied, true);

    const vueRule = rules[1] as Record<string, unknown>;
    const oneOf = vueRule.oneOf as Array<Record<string, unknown>>;

    // Find the CSS fallback (matches lang=css)
    const cssFallback = oneOf.find((r) => {
      const rq = r.resourceQuery as RegExp;
      return rq && rq.test("vue&type=style&index=0&lang=css");
    });
    assert.ok(cssFallback, "should have a CSS fallback rule");

    const use = cssFallback!.use as Array<Record<string, unknown>>;
    assert.equal(use.length, 2, "CSS fallback should have scope-loader + style-loader");
    assert.equal(use[0].loader, "@vizejs/rspack-plugin/scope-loader");
    assert.equal(use[1].loader, "@vizejs/rspack-plugin/style-loader");
  });
});
