import assert from "node:assert/strict";
import { createRequire } from "node:module";
import {
  browserRuntime,
  captureHashes,
  dataUrl,
  hash,
  relink,
} from "./native-scoped-ssr-reference.ts";

export async function hydrate(fixtures: any[], rendered: any[]) {
  const fromTests = createRequire(new URL("../../package.json", import.meta.url));
  const { chromium } = fromTests("@playwright/test");
  const browser = await chromium.launch();
  const runtimeUrl = browserRuntime();
  const executions = [];
  try {
    for (const [index, fixture] of fixtures.entries()) {
      const actual = rendered[index];
      assert.deepEqual(actual.hashes, captureHashes(actual));
      assert.equal(actual.htmlSha256, hash(actual.html));
      const page = await browser.newPage();
      const errors: string[] = [];
      page.on("pageerror", (error: Error) => errors.push(error.message));
      await page.setContent("<!doctype html><html><head></head><body></body></html>");
      const clientUrl = dataUrl(relink(fixture.stockClientModule, { vue: runtimeUrl }));
      const execution = await page.evaluate(
        async ({ actual, fixture, runtimeUrl, clientUrl }: any) => {
          const runtime = await import(runtimeUrl);
          const client = await import(clientUrl);
          const style = document.createElement("style");
          style.textContent = actual.css;
          document.head.append(style);
          const consumedCss = style.textContent;
          const mount = document.createElement("section");
          mount.innerHTML = actual.html;
          const outside = document.createElement("p");
          outside.className = fixture.classname;
          document.body.append(mount, outside);
          const before = [...mount.querySelectorAll("*")];
          const record = () =>
            [...mount.querySelectorAll("*")].map((el) => ({
              tag: el.tagName,
              scope: el.getAttribute(actual.scopeId),
              classname: el.className,
              text: el.textContent,
            }));
          const beforeHydration = record();
          const warnings: string[] = [];
          // Client rendering is the separately verified pinned stock oracle;
          // the server module/CSS/HTML above must come from fresh native Rust.
          const app = runtime.createSSRApp(
            { render: client.render, __scopeId: actual.scopeId },
            fixture.props,
          );
          app.config.warnHandler = (message: string) => warnings.push(message);
          app.mount(mount);
          await runtime.nextTick();
          const after = [...mount.querySelectorAll("*")];
          if (before.length !== after.length || before.some((el, at) => el !== after[at]))
            throw Error("hydration replaced original nodes");
          if (after.some((el) => !el.hasAttribute(actual.scopeId)))
            throw Error("missing nested scope");
          if (warnings.length) throw Error(warnings.join("|"));
          const afterHydration = record();
          const empty =
            after.find((el) => el.classList.contains(fixture.classname) && el.matches(":empty")) ??
            after.find((el) => el.tagName === "P" && el.matches(":empty"));
          if (!empty) throw Error("missing original empty element");
          const classControl = empty.classList.contains(fixture.classname)
            ? "existing root attrs.class fallthrough"
            : "direct browser class mutation; no authored class admission";
          empty.className = fixture.classname;
          const states = [];
          for (const text of ["", "visible", ""]) {
            empty.textContent = text;
            states.push({
              text: empty.textContent,
              empty: empty.matches(":empty"),
              scoped: empty.matches(`.${fixture.classname}[${actual.scopeId}]:empty`),
              background: getComputedStyle(empty).backgroundColor,
            });
          }
          const outsideState = {
            scope: outside.getAttribute(actual.scopeId),
            background: getComputedStyle(outside).backgroundColor,
          };
          app.unmount();
          const afterUnmount = mount.childElementCount;
          mount.remove();
          outside.remove();
          const negative = document.createElement("section");
          negative.innerHTML = fixture.metadataOnlyHtml;
          document.body.append(negative);
          const metadataOnly = runtime.createSSRApp(
            { render: client.render, __scopeId: actual.scopeId },
            fixture.props,
          );
          metadataOnly.mount(negative);
          await runtime.nextTick();
          const metadataOnlyAfterHydration = [...negative.querySelectorAll("*")].map((el) => ({
            scope: el.getAttribute(actual.scopeId),
            background: getComputedStyle(el).backgroundColor,
          }));
          metadataOnly.unmount();
          negative.remove();
          style.remove();
          return {
            id: fixture.id,
            runtime: runtime.version,
            scopeId: actual.scopeId,
            nativeHashes: actual.hashes,
            htmlSha256: actual.htmlSha256,
            consumedCss,
            loadedClientUrl: clientUrl,
            classControl,
            beforeHydration,
            afterHydration,
            reusedEveryElement: true,
            hydrationWarnings: warnings,
            states,
            outsideState,
            afterUnmount,
            metadataOnlyAfterHydration,
          };
        },
        { actual, fixture, runtimeUrl, clientUrl },
      );
      assert.deepEqual(errors, []);
      assert.equal(execution.runtime, "3.5.35");
      assert.equal(execution.scopeId, fixture.scopeId);
      assert.deepEqual(execution.nativeHashes, captureHashes(actual));
      assert.equal(execution.htmlSha256, hash(actual.html));
      assert.equal(hash(execution.consumedCss), actual.hashes.cssSha256);
      assert.equal(execution.loadedClientUrl, clientUrl);
      const { consumedCss, loadedClientUrl, ...receipt } = execution;
      const hashes = {
        cssSha256: hash(consumedCss),
        clientModuleSha256: hash(fixture.stockClientModule),
        loadedClientSha256: hash(loadedClientUrl),
      };
      assert.deepEqual(execution.beforeHydration, execution.afterHydration);
      assert(execution.afterHydration.every((el: any) => el.scope === ""));
      assert.deepEqual(execution.hydrationWarnings, []);
      assert.equal(execution.reusedEveryElement, true);
      assert.equal(
        execution.classControl,
        index === 0
          ? "existing root attrs.class fallthrough"
          : "direct browser class mutation; no authored class admission",
      );
      for (const [at, state] of execution.states.entries()) {
        const empty = at !== 1;
        assert.equal(state.text, empty ? "" : "visible");
        assert.equal(state.empty, empty);
        assert.equal(state.scoped, empty);
        assert.equal(state.background, empty ? "rgb(0, 0, 255)" : "rgba(0, 0, 0, 0)");
      }
      assert.deepEqual(execution.outsideState, { scope: null, background: "rgba(0, 0, 0, 0)" });
      assert.equal(execution.afterUnmount, 0);
      assert(
        execution.metadataOnlyAfterHydration.every(
          (el: any) => el.scope === null && el.background === "rgba(0, 0, 0, 0)",
        ),
      );
      executions.push({ ...receipt, consumedHashes: hashes });
      await page.close();
    }
    return {
      runtime: "vue@3.5.35",
      browser: browser.version(),
      currentSource: true,
      clientAuthority: "independent pinned stock original-SFC client oracle",
      executions,
    };
  } finally {
    await browser.close();
  }
}
