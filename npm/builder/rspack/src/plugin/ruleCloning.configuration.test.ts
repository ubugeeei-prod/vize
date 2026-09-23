import { describe, test } from "node:test";
import assert from "node:assert/strict";
import "./../test/setup.ts";
import { applyRuleCloning } from "./ruleCloning.ts";

void describe("applyRuleCloning configuration", () => {
  void test("shares file filters while keeping request conditions and metadata on the main branch", () => {
    const include = /src/;
    const exclude = /vendor/;
    const issuer = /entry/;
    const resourceQuery = { not: [/raw/] };
    const options = { ssr: true };
    const rules = [
      {
        test: /\.vue$/,
        include,
        exclude,
        issuer,
        resourceQuery,
        sideEffects: false,
        loader: "@vizejs/rspack-plugin/loader",
        options,
      },
    ];
    applyRuleCloning(rules, true);
    const rule = rules[0] as Record<string, unknown>;
    assert.equal(rule.include, include);
    assert.equal(rule.exclude, exclude);
    assert.equal(rule.issuer, undefined);
    assert.equal(rule.resourceQuery, undefined);
    assert.equal(rule.sideEffects, undefined);
    assert.equal(rule.loader, undefined);
    assert.equal(rule.options, undefined);
    const branches = rule.oneOf as Array<Record<string, unknown>>;
    const main = branches.at(-1)!;
    assert.equal(main.issuer, issuer);
    assert.equal(main.resourceQuery, resourceQuery);
    assert.equal(main.sideEffects, false);
    const uses = main.use as Array<{ options: unknown }>;
    assert.deepEqual(uses[0].options, { ssr: true, css: { native: true } });
    for (const style of branches.slice(0, -1)) {
      assert.equal(style.sideEffects, true);
    }
    assert.deepEqual(options, { ssr: true });
  });
  void test("handles resolved file paths for vize loader", () => {
    const rules = [
      { test: /\.css$/, use: ["css-loader"] },
      {
        test: /\.vue$/,
        use: [
          {
            loader: "/path/to/node_modules/@vizejs/rspack-plugin/dist/loader/index.mjs",
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

    // Last entry (main loader fallback) should carry the resolved native CSS mode
    const mainBranch = oneOf[oneOf.length - 1];
    const use = mainBranch.use as Array<Record<string, unknown>>;
    assert.deepEqual(use[0], {
      loader: "@vizejs/rspack-plugin/loader",
      options: { css: { native: true } },
    });

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

    // Should preserve existing options and carry the resolved native CSS mode
    assert.equal(use[0].loader, "@vizejs/rspack-plugin/loader");
    assert.deepEqual(use[0].options, {
      sourceMap: true,
      css: { native: true },
    });
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
