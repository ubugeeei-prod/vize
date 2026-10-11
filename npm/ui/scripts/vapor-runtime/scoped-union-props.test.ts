import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createRequire, stripTypeScriptTypes } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";

import { compileSfc } from "@vizejs/native";

import { installDomGlobals, withoutBrowserGlobals } from "./dom-env.ts";

const require = createRequire(import.meta.url);
const window = installDomGlobals();
const stableCompiler = require("vue/compiler-sfc") as typeof import("vue/compiler-sfc");
const supportedCompiler =
  require("vue-vapor-runtime-supported/compiler-sfc") as typeof stableCompiler;
const root = new URL(
  "../../../../tests/_fixtures/differential/compiler/scoped-union-props/",
  import.meta.url,
);
const packet = JSON.parse(readFileSync(new URL("fixture.json", root), "utf8")) as {
  serverVue: string;
  supportedVue: string;
  inputs: { path: string; sha256: string }[];
  initialProps: Record<string, unknown>;
  updatedProps: Record<string, unknown>;
  expected: { initial: string; updated: string; unmounted: string };
};
const digest = (value: string): string => createHash("sha256").update(value).digest("hex");
let sequence = 0;

function originalInput(name: string): string {
  const source = readFileSync(new URL(name, root), "utf8");
  assert.equal(digest(source), packet.inputs.find((input) => input.path === name)?.sha256);
  return source;
}

function compileWithVue(
  compiler: typeof stableCompiler,
  source: string,
  filename: string,
  lane: "dom" | "ssr" | "vapor",
): string {
  const { descriptor, errors } = compiler.parse(source, { filename });
  assert.deepEqual(errors, []);
  return compiler.compileScript(descriptor, {
    id: "scoped-union-props",
    inlineTemplate: true,
    fs: {
      fileExists: (name: string) => {
        try {
          readFileSync(name);
          return true;
        } catch {
          return false;
        }
      },
      readFile: (name: string) => readFileSync(name, "utf8"),
    },
    templateOptions: { ssr: lane === "ssr" },
    vapor: lane === "vapor",
  } as Parameters<typeof compiler.compileScript>[1]).content;
}

async function evaluate(
  source: string,
  vueUrl: string,
  serverUrl?: string,
): Promise<import("vue").Component> {
  let code = stripTypeScriptTypes(source, { mode: "strip" }).replaceAll(
    /from ["']vue["']/g,
    `from ${JSON.stringify(vueUrl)}`,
  );
  if (serverUrl !== undefined) {
    code = code.replaceAll(
      /from ["'](?:@vue\/server-renderer|vue\/server-renderer)["']/g,
      `from ${JSON.stringify(serverUrl)}`,
    );
  }
  return (
    await import(
      `data:text/javascript;base64,${Buffer.from(`${code}\n// ${sequence++}`).toString("base64")}`
    )
  ).default as import("vue").Component;
}

await test("whole inline and imported union props retain names and Boolean casting in SSR, DOM and Vapor", async () => {
  const work = mkdtempSync(path.join(tmpdir(), "vize-scoped-union-props-"));
  const observations: unknown[] = [];
  writeFileSync(path.join(work, "Props.ts"), originalInput("Props.ts.txt"));
  try {
    assert.equal(stableCompiler.version, packet.serverVue);
    assert.equal(supportedCompiler.version, packet.supportedVue);
    const runtimeUrl = pathToFileURL(
      require.resolve("vue-vapor-runtime-supported/dist/vue.runtime-with-vapor.esm-browser.js"),
    ).href;
    const client = (await import(runtimeUrl)) as typeof import("vue");
    assert.equal(client.version, packet.supportedVue);

    for (const name of ["Inline", "Imported", "Plain"]) {
      const source = originalInput(`${name}.vue.txt`);
      const filename = path.join(work, `${name}.vue`);
      writeFileSync(filename, source);
      const native: Record<string, string> = {};
      for (const lane of ["ssr", "dom", "vapor"] as const) {
        const result = compileSfc(source, {
          filename,
          mode: "module",
          isTs: true,
          ssr: lane === "ssr",
          vapor: lane === "vapor",
        });
        assert.deepEqual(result.errors, []);
        native[lane] = result.code;
      }

      for (const provider of ["vue", "vue-vapor-runtime-supported"]) {
        const vueRequire = createRequire(require.resolve(`${provider}/package.json`));
        const vueUrl = pathToFileURL(vueRequire.resolve("vue/dist/vue.cjs.js")).href;
        const serverRequire = createRequire(vueRequire.resolve("vue/server-renderer"));
        const serverUrl = pathToFileURL(serverRequire.resolve("@vue/server-renderer")).href;
        const serverVue = (await import(vueUrl)) as typeof import("vue");
        const renderer = (await import(serverUrl)) as typeof import("vue/server-renderer");
        const compiler = provider === "vue" ? stableCompiler : supportedCompiler;
        assert.equal(serverVue.version, compiler.version);
        for (const backend of ["native", "vue"]) {
          const code =
            backend === "native" ? native.ssr! : compileWithVue(compiler, source, filename, "ssr");
          const component = await evaluate(code, vueUrl, serverUrl);
          const html: string[] = [];
          for (const [phase, props] of [
            ["initial", packet.initialProps],
            ["updated", { ...packet.initialProps, ...packet.updatedProps }],
          ] as const) {
            const app = serverVue.createSSRApp({ render: () => serverVue.h(component, props) });
            const problems: string[] = [];
            app.config.warnHandler = (message) => problems.push(message);
            app.config.errorHandler = (error) => problems.push(String(error));
            const actual = await withoutBrowserGlobals(() => renderer.renderToString(app));
            assert.equal(actual, packet.expected[phase], `${name}/${provider}/${backend}/${phase}`);
            assert.deepEqual(problems, []);
            html.push(actual);
          }
          observations.push({
            name,
            provider: serverVue.version,
            backend,
            lane: "ssr",
            sourceSha256: digest(source),
            moduleSha256: digest(code),
            html,
          });
        }
      }

      for (const lane of ["dom", "vapor"] as const) {
        for (const backend of ["native", "vue"]) {
          const code =
            backend === "native"
              ? native[lane]!
              : compileWithVue(supportedCompiler, source, filename, lane);
          const component = await evaluate(code, runtimeUrl);
          for (const mode of ["mount", "hydrate"]) {
            const host = window.document.createElement("div");
            window.document.body.append(host);
            if (mode === "hydrate") host.innerHTML = packet.expected.initial;
            const originalRoot = host.firstChild;
            const state = client.reactive({ ...packet.initialProps });
            const wrapper = { render: () => client.h(component, { ...state }) };
            const app =
              mode === "hydrate" ? client.createSSRApp(wrapper) : client.createApp(wrapper);
            let disposed = false;
            const problems: string[] = [];
            app.config.warnHandler = (message) => problems.push(message);
            app.config.errorHandler = (error) => problems.push(String(error));
            app.use(
              (client as unknown as { vaporInteropPlugin: import("vue").Plugin })
                .vaporInteropPlugin,
            );
            try {
              app.mount(host);
              await client.nextTick();
              assert.equal(
                host.innerHTML,
                packet.expected.initial,
                `${name}/${lane}/${backend}/${mode}`,
              );
              if (mode === "hydrate") assert.equal(host.firstChild, originalRoot);
              const mountedRoot = host.firstChild;
              Object.assign(state, packet.updatedProps);
              await client.nextTick();
              assert.equal(host.innerHTML, packet.expected.updated);
              assert.equal(host.firstChild, mountedRoot);
              assert.deepEqual(problems, []);
              app.unmount();
              disposed = true;
              assert.equal(host.innerHTML, packet.expected.unmounted);
              assert.deepEqual(problems, []);
              observations.push({
                name,
                provider: client.version,
                backend,
                lane,
                mode,
                sourceSha256: digest(source),
                moduleSha256: digest(code),
                initial: packet.expected.initial,
                updated: packet.expected.updated,
                unmounted: host.innerHTML,
              });
            } finally {
              if (!disposed) app.unmount();
              host.remove();
            }
          }
        }
      }
    }
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
  assert.equal(observations.length, 36);
  console.log(
    JSON.stringify({ check: "scoped union prop whole SSR/DOM/Vapor corpus", observations }),
  );
});
