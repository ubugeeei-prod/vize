import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

void test("Nuxt module entry avoids import.meta syntax in loader-facing sources", () => {
  const fixtures = [
    ["src/index.ts", new URL("./index.ts", import.meta.url)],
    ["dist/index.mjs", new URL("../dist/index.mjs", import.meta.url)],
  ] as const;

  const offsetsByFile = fixtures.map(([name, url]) => [
    name,
    importMetaOffsets(fs.readFileSync(url, "utf8")),
  ]);

  assert.deepEqual(offsetsByFile, [
    ["src/index.ts", []],
    ["dist/index.mjs", []],
  ]);
});

void test("Nuxt module entry runs in a Nuxt 2 webpack-style context", async () => {
  const { default: nuxtModule } = await import(new URL("../dist/index.mjs", import.meta.url).href);
  const hookNames: string[] = [];
  const nuxt: {
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
      hookNames: [],
      requiredModules: { "@vizejs/nuxt": true },
      vite: undefined,
    },
  );
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
