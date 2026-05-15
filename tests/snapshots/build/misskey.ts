import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { execSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { misskeyApp, VIZE_BIN } from "../../_helpers/apps.ts";
import { assertParsesAsModule } from "../../_helpers/assertions.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const app = misskeyApp;

describe(`${app.name} build (compiler)`, () => {
  before(() => {
    if (!fs.existsSync(VIZE_BIN)) {
      console.log(`Skipping: vize binary not found at ${VIZE_BIN}`);
      process.exit(0);
    }
    if (app.setup) app.setup();
  });

  it("vize build compiles without errors", () => {
    const cwd = app.check!.cwd;
    const patterns = app.check!.patterns.map((p) => `'${p}'`).join(" ");
    const outDir = path.join(__dirname, "__snapshots__", `${app.name}-build-output`);
    fs.rmSync(outDir, { recursive: true, force: true });

    const cmd = `${VIZE_BIN} build ${patterns} -o '${outDir}' --continue-on-error`;
    console.log(`Running: ${cmd}`);

    const stdout = execSync(cmd, {
      cwd,
      timeout: 120_000,
      maxBuffer: 100 * 1024 * 1024,
    }).toString();

    console.log(stdout);

    assert.ok(fs.existsSync(outDir), "output directory should exist");
    const jsFiles = fs
      .readdirSync(outDir, { recursive: true })
      .filter((f) => String(f).endsWith(".js"));
    console.log(`Generated ${jsFiles.length} JS files`);
    assert.ok(jsFiles.length > 0, "should produce .js output files");
  });

  it("compiled output is valid JavaScript", () => {
    const outDir = path.join(__dirname, "__snapshots__", `${app.name}-build-output`);
    if (!fs.existsSync(outDir)) {
      assert.fail("output directory does not exist - run build test first");
    }

    const jsFiles = fs
      .readdirSync(outDir, { recursive: true })
      .filter((f) => String(f).endsWith(".js"))
      .slice(0, 10);

    for (const entry of jsFiles) {
      const file = String(entry);
      const filePath = path.join(outDir, file);
      const content = fs.readFileSync(filePath, "utf-8");
      assertParsesAsModule(content, file);
      console.log(`Valid: ${file}`);
    }
  });

  it("preserves misskey slot outlets, directives, and built-ins", () => {
    const outDir = path.join(__dirname, "__snapshots__", `${app.name}-build-output`);
    if (!fs.existsSync(outDir)) {
      assert.fail("output directory does not exist - run build test first");
    }

    const readOutput = (file: string) => {
      const filePath = path.join(outDir, file);
      assert.ok(fs.existsSync(filePath), `missing compiled file: ${file}`);
      return fs.readFileSync(filePath, "utf-8");
    };

    const assertRenderSlot = (file: string, slotName: string) => {
      const content = readOutput(file);
      assert.ok(
        content.includes(`_renderSlot(_ctx.$slots, "${slotName}"`),
        `${file} should render slot "${slotName}"`,
      );
      assert.ok(
        !content.includes(`_createElementBlock("slot"`),
        `${file} should not emit literal <slot> elements`,
      );
      assert.ok(
        !content.includes(`_createElementVNode("slot"`),
        `${file} should not emit literal <slot> vnodes`,
      );
    };

    const assertTooltipDirective = (file: string) => {
      const content = readOutput(file);
      const normalized = content.replace(/\s+/g, " ");
      assert.ok(
        content.includes(`const _directive_tooltip = _resolveDirective("tooltip")`),
        `${file} should resolve the tooltip directive`,
      );
      assert.ok(
        content.includes("_withDirectives("),
        `${file} should apply tooltip with withDirectives`,
      );
      assert.ok(
        /\[\[\s*_directive_tooltip\b/.test(normalized),
        `${file} should pass tooltip directive bindings`,
      );
    };

    assertRenderSlot("MkLazy.js", "default");
    assertRenderSlot("MkPaginationControl.js", "default");
    assertRenderSlot("MkTl.js", "left");
    assertRenderSlot("PageWithHeader.js", "default");

    assertTooltipDirective("MkSwitch.js");
    assertTooltipDirective("MkPageHeader.tabs.js");

    const routerView = readOutput("RouterView.js");
    assert.ok(
      routerView.includes("_createVNode(_KeepAlive") &&
        routerView.includes("_createVNode(_Suspense"),
      "RouterView.js should preserve KeepAlive and Suspense built-ins",
    );

    const streaming = readOutput("MkStreamingNotesTimeline.js");
    assert.ok(
      streaming.includes(`_renderSlot(_ctx.$slots, "empty"`),
      "MkStreamingNotesTimeline.js should preserve the empty slot fallback",
    );
    assert.ok(
      streaming.includes(`const _directive_appear = _resolveDirective("appear")`),
      "MkStreamingNotesTimeline.js should resolve the appear directive",
    );
    assert.ok(
      streaming.includes('_withDirectives(_createElementVNode("button"'),
      "MkStreamingNotesTimeline.js should wrap the load-more button with directives",
    );
    assert.ok(
      streaming.includes(
        "[[_directive_appear, _unref(prefer).s.enableInfiniteScroll ? _unref(paginator).fetchOlder : null], [_vShow, _unref(paginator).canFetchOlder.value]]",
      ),
      "MkStreamingNotesTimeline.js should keep appear and v-show on the same button",
    );

    const pagination = readOutput("MkPagination.js");
    const normalizedPagination = pagination.replace(/\s+/g, " ");
    assert.ok(
      pagination.includes(`const _directive_appear = _resolveDirective("appear")`),
      "MkPagination.js should resolve the appear directive",
    );
    assert.ok(
      /_withDirectives\(\(_openBlock\(\), _createBlock\(\s*MkButton\b/.test(normalizedPagination),
      "MkPagination.js should apply appear via withDirectives on load-more buttons",
    );
    assert.ok(
      normalizedPagination.includes(
        "[[_directive_appear, shouldEnableInfiniteScroll.value ? upButtonClick : null]]",
      ),
      "MkPagination.js should keep the appear binding on the upward load-more button",
    );
  });
});
