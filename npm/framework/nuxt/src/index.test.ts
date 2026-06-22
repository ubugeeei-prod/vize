import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

const NUXT2_SAFE_KIT_VERSION = "3.11.2";

void test("Nuxt module entry avoids loader-unsafe syntax and static kit imports", () => {
  const fixtures = [
    ["src/index.ts", new URL("./index.ts", import.meta.url)],
    ["dist/index.mjs", new URL("../dist/index.mjs", import.meta.url)],
    ["src/resolver.ts", new URL("./resolver.ts", import.meta.url)],
  ] as const;

  const offsetsByFile = fixtures.map(([name, url]) => {
    const source = fs.readFileSync(url, "utf8");
    assert.doesNotMatch(source, /from\s+["']@nuxt\/kit["']/);
    return [name, importMetaOffsets(source)];
  });

  assert.deepEqual(offsetsByFile, [
    ["src/index.ts", []],
    ["dist/index.mjs", []],
    ["src/resolver.ts", []],
  ]);
});

void test("Nuxt module entry runs in a Nuxt 2 webpack-style context", async () => {
  const { default: nuxtModule } = await import(new URL("../dist/index.mjs", import.meta.url).href);
  const hookNames: string[] = [];
  const nuxt: {
    _version: string;
    options: Record<string, unknown> & {
      rootDir: string;
      builder: string;
      build: { publicPath: string };
      router: { base: string };
      modules: unknown[];
      buildDir: string;
      dev: boolean;
    };
    hook(name: string, callback: (...args: unknown[]) => unknown): void;
  } = {
    _version: "2.17.3",
    options: {
      rootDir: process.cwd(),
      builder: "webpack",
      build: { publicPath: "/_nuxt/" },
      router: { base: "/docs/" },
      modules: [],
      buildDir: ".nuxt",
      dev: false,
    },
    hook(name: string, callback: (...args: unknown[]) => unknown) {
      assert.equal(typeof callback, "function");
      hookNames.push(name);
    },
  };

  await nuxtModule(
    {
      compiler: false,
      musea: false,
      compatibility: { nuxtVersion: 2, vueVersion: 2 },
    },
    nuxt,
  );

  assert.deepEqual(await nuxtModule.getMeta(), {
    name: "@vizejs/nuxt",
    configKey: "vize",
  });
  assert.deepEqual(
    {
      hookNames,
      requiredModules: nuxt.options._requiredModules,
      vite: nuxt.options.vite,
    },
    {
      hookNames: ["close", "builder:prepared", "build:templates"],
      requiredModules: { "@vizejs/nuxt": true },
      vite: undefined,
    },
  );
});

void test("packed Nuxt module depends on the Nuxt 2-safe kit line", () => {
  const packageRoot = new URL("..", import.meta.url);
  const packDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-pack-"));

  try {
    execFileSync("pnpm", ["pack", "--pack-destination", packDir], {
      cwd: packageRoot,
      stdio: "pipe",
    });

    const tarballs = fs.readdirSync(packDir).filter((name) => name.endsWith(".tgz"));
    assert.equal(tarballs.length, 1);

    const packedPackageJson = JSON.parse(
      execFileSync("tar", ["-xOf", path.join(packDir, tarballs[0]), "package/package.json"], {
        encoding: "utf8",
      }),
    ) as { dependencies?: Record<string, string> };
    const packedKitVersion = packedPackageJson.dependencies?.["@nuxt/kit"];

    assert.equal(packedKitVersion, NUXT2_SAFE_KIT_VERSION);
    assert.ok(
      !packedKitVersion?.startsWith("4."),
      "Nuxt 2 must not load @nuxt/kit 4.x through @vizejs/nuxt",
    );
  } finally {
    fs.rmSync(packDir, { recursive: true, force: true });
  }
});

function importMetaOffsets(source: string): string[] {
  const offsets: string[] = [];
  let index = 0;
  let line = 1;
  let column = 1;
  let state: "code" | "line-comment" | "block-comment" | "single" | "double" | "template" = "code";
  let escaped = false;

  while (index < source.length) {
    const char = source[index];
    const next = source[index + 1];
    const currentLine = line;
    const currentColumn = column;

    if (state === "code") {
      if (char === "/" && next === "/") {
        advance(2);
        state = "line-comment";
        continue;
      }
      if (char === "/" && next === "*") {
        advance(2);
        state = "block-comment";
        continue;
      }
      if (char === "'") {
        advance(1);
        escaped = false;
        state = "single";
        continue;
      }
      if (char === '"') {
        advance(1);
        escaped = false;
        state = "double";
        continue;
      }
      if (char === "`") {
        advance(1);
        escaped = false;
        state = "template";
        continue;
      }
      if (
        source.startsWith("import.meta", index) &&
        isIdentifierBoundary(source[index - 1]) &&
        isIdentifierBoundary(source[index + "import.meta".length])
      ) {
        offsets.push(`${currentLine}:${currentColumn}`);
      }
      advance(1);
      continue;
    }

    if (state === "line-comment") {
      advance(1);
      if (char === "\n") {
        state = "code";
      }
      continue;
    }

    if (state === "block-comment") {
      if (char === "*" && next === "/") {
        advance(2);
        state = "code";
        continue;
      }
      advance(1);
      continue;
    }

    if (escaped) {
      advance(1);
      escaped = false;
      continue;
    }

    if (char === "\\") {
      advance(1);
      escaped = true;
      continue;
    }

    if (
      (state === "single" && char === "'") ||
      (state === "double" && char === '"') ||
      (state === "template" && char === "`")
    ) {
      advance(1);
      state = "code";
      continue;
    }

    advance(1);
  }

  return offsets;

  function advance(count: number) {
    for (let i = 0; i < count; i++) {
      const consumed = source[index];
      index++;
      if (consumed === "\n") {
        line++;
        column = 1;
      } else {
        column++;
      }
    }
  }
}

function isIdentifierBoundary(char: string | undefined): boolean {
  return char === undefined || !/[A-Za-z0-9_$]/.test(char);
}
