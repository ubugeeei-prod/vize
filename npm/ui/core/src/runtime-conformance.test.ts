import assert from "node:assert/strict";
import { readdir } from "node:fs/promises";
import path from "node:path";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, type VNode } from "vue";
import { renderToString } from "vue/server-renderer";

import ActionButton from "./action-button.vue";
import CheckboxControl from "./checkbox-control.vue";
import IdProvider from "./deterministic-id-provider.vue";
import { useDeterministicId } from "./deterministic-id.ts";
import PrimitiveElement from "./primitive-element.vue";
import VisuallyHidden from "./visually-hidden.vue";

interface RuntimeFixture {
  /** Stable name included in assertion diagnostics. */
  readonly name: string;
  /** Canonical SFC whose SSR and hydration behavior this fixture covers. */
  readonly sourceFile: string;
  /** Build a fresh vnode so no request can inherit another request's state. */
  readonly render: () => VNode;
  /** Assert server output semantics before the browser repairs or normalizes DOM. */
  readonly assertServerMarkup: (html: string) => void;
  /** Assert hydrated accessibility semantics in a browser-like DOM. */
  readonly assertHydratedDom: (host: HTMLElement) => void;
}

const DeterministicIdProbe = defineComponent({
  name: "RuntimeConformanceDeterministicIdProbe",
  setup() {
    const id = useDeterministicId({ hint: "control" });
    return () => h("input", { id: id.value, "aria-label": "Email" });
  },
});

const runtimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "button",
    sourceFile: "action-button.vue",
    render: () =>
      h(
        ActionButton,
        { loading: true },
        {
          default: () => "Save changes",
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /<button/);
      assert.match(html, /aria-busy="true"/);
      assert.match(html, /data-state="loading"/);
      assert.match(html, /Save changes/);
      assert.match(html, /<\/button>/);
    },
    assertHydratedDom(host) {
      const button = host.querySelector('[data-vize-ui="button"]');
      assert.ok(button instanceof HTMLButtonElement);
      assert.equal(button.textContent, "Save changes");
      assert.equal(button.getAttribute("aria-busy"), "true");
    },
  },
  {
    name: "checkbox",
    sourceFile: "checkbox-control.vue",
    render: () =>
      h(CheckboxControl, {
        ariaLabel: "Accept terms",
        defaultChecked: true,
      }),
    assertServerMarkup(html) {
      assert.match(html, /type="checkbox"/);
      assert.match(html, /aria-label="Accept terms"/);
      assert.match(html, /aria-checked="true"/);
      assert.match(html, /checked/);
    },
    assertHydratedDom(host) {
      const checkbox = host.querySelector('[data-vize-ui="checkbox"]');
      assert.ok(checkbox instanceof HTMLInputElement);
      assert.equal(checkbox.checked, true);
      assert.equal(checkbox.getAttribute("aria-checked"), "true");
    },
  },
  {
    name: "deterministic-id-provider",
    sourceFile: "deterministic-id-provider.vue",
    render: () =>
      h(
        IdProvider,
        { prefix: "form", seed: "runtime" },
        {
          default: () => h(DeterministicIdProbe),
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /id="form-runtime-control-0"/);
      assert.match(html, /aria-label="Email"/);
    },
    assertHydratedDom(host) {
      const input = host.querySelector("input");
      assert.ok(input instanceof HTMLInputElement);
      assert.equal(input.id, "form-runtime-control-0");
      assert.equal(input.getAttribute("aria-label"), "Email");
    },
  },
  {
    name: "primitive",
    sourceFile: "primitive-element.vue",
    render: () =>
      h(
        PrimitiveElement,
        { as: "section" },
        {
          default: () => "Composable content",
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /^<section/);
      assert.match(html, /Composable content/);
      assert.match(html, /<\/section>$/);
    },
    assertHydratedDom(host) {
      const primitive = host.querySelector('[data-vize-ui="primitive"]');
      assert.ok(primitive instanceof HTMLElement);
      assert.equal(primitive.tagName, "SECTION");
    },
  },
  {
    name: "visually-hidden",
    sourceFile: "visually-hidden.vue",
    render: () =>
      h(VisuallyHidden, null, {
        default: () => h("button", { type: "button" }, "Dismiss notification"),
      }),
    assertServerMarkup(html) {
      assert.match(html, /^<span/);
      assert.match(html, /data-vize-ui="visually-hidden"/);
      assert.match(html, /<button type="button">Dismiss notification<\/button>/);
    },
    assertHydratedDom(host) {
      const control = host.querySelector("button");
      assert.ok(control instanceof HTMLButtonElement);
      assert.equal(control.textContent, "Dismiss notification");
    },
  },
];

/** Recursively collect canonical SFC paths relative to the package source root. */
async function collectSourceSfcFiles(
  directory: string,
  relativeDirectory = "",
): Promise<readonly string[]> {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry): Promise<readonly string[]> => {
      const entryPath = path.join(directory, entry.name);
      const relativePath = path.join(relativeDirectory, entry.name);
      if (entry.isDirectory()) return collectSourceSfcFiles(entryPath, relativePath);
      return entry.isFile() && entry.name.endsWith(".vue") ? [relativePath] : [];
    }),
  );
  return files.flat().sort((left, right) => left.localeCompare(right));
}

/** Create an isolated application root for one renderer request. */
function createFixtureRoot(fixture: RuntimeFixture) {
  return defineComponent({
    name: `RuntimeConformance${fixture.name}`,
    setup: () => fixture.render,
  });
}

/** Render with a fresh SSR application so request-local state cannot be reused. */
async function renderFixture(fixture: RuntimeFixture): Promise<string> {
  return renderToString(createSSRApp(createFixtureRoot(fixture)));
}

test("declares an SSR and hydration fixture for every source SFC", async () => {
  const sourceFiles = await collectSourceSfcFiles(path.resolve("src"));
  const fixtureFiles = runtimeFixtures
    .map((fixture) => fixture.sourceFile)
    .sort((left, right) => left.localeCompare(right));

  assert.deepEqual(fixtureFiles, sourceFiles);
});

test("renders stable, accessible markup across isolated SSR requests", async () => {
  for (const fixture of runtimeFixtures) {
    const [left, right] = await Promise.all([renderFixture(fixture), renderFixture(fixture)]);
    assert.equal(left, right, `${fixture.name} emitted request-dependent SSR markup`);
    fixture.assertServerMarkup(left);
  }
});

test("hydrates every shipped component without warnings or node replacement", async () => {
  for (const fixture of runtimeFixtures) {
    const serverHtml = await renderFixture(fixture);
    const host = document.createElement("div");
    host.innerHTML = serverHtml;
    document.body.append(host);
    const serverRoot = host.firstElementChild;
    assert.ok(serverRoot, `${fixture.name} did not emit a root element`);

    const diagnostics: string[] = [];
    const originalWarn = console.warn;
    const originalError = console.error;
    console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
    console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
    const app = createSSRApp(createFixtureRoot(fixture));
    let mounted = false;

    try {
      app.mount(host);
      mounted = true;
      assert.ok(
        host.firstElementChild === serverRoot,
        `${fixture.name} replaced its server-rendered root during hydration`,
      );
      assert.deepEqual(diagnostics, [], `${fixture.name} emitted hydration diagnostics`);
      fixture.assertHydratedDom(host);
    } finally {
      if (mounted) app.unmount();
      host.remove();
      console.warn = originalWarn;
      console.error = originalError;
    }
  }
});
