import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { JsconfigOracle } from "./support/typecheck/cli-jsconfig-runtime.ts";
import type { ProcessRow } from "./support/typecheck/cli-jsconfig-runtime.ts";

const options = {
  allowJs: true,
  checkJs: true,
  maxNodeModuleJsDepth: 2,
  module: "ESNext",
  moduleResolution: "bundler",
  noEmit: true,
  skipLibCheck: true,
  strict: true,
  target: "ES2022",
  types: [],
};
const message = "Type 'number' is not assignable to type 'string'.";
const emptyReport = { files: [], programs: [], errorCount: 0, warningCount: 0, fileCount: 0 };

function report(valid: boolean, compilerOptions = options): unknown {
  return {
    files: [
      { file: "src/message.js", diagnostics: valid ? [] : [`error:2:14 [TS2322] ${message}`] },
    ],
    programs: [
      {
        root: ".",
        tsconfig: "jsconfig.json",
        compilerOptions,
        files: ["src/message.js"],
      },
    ],
    errorCount: valid ? 0 : 1,
    warningCount: 0,
    fileCount: 1,
  };
}

function whole(row: ProcessRow, status: number, expected: unknown, stderr = ""): void {
  assert.equal(row.status, status, row.stderr || row.stdout);
  assert.equal(row.stderr, stderr);
  assert.deepEqual(JSON.parse(row.stdout), expected);
  assert.equal(row.stdout, `${JSON.stringify(expected, null, 2)}\n`);
}

function stock(row: ProcessRow, file: string, valid: boolean, line = 2): void {
  assert.equal(row.status, valid ? 0 : 1, row.stderr || row.stdout);
  assert.equal(row.stderr, "");
  assert.equal(row.stdout, valid ? "" : `${file}(${line},14): error TS2322: ${message}\n`);
}

function prepare(oracle: JsconfigOracle, prefix = ""): void {
  oracle.write(`${prefix}package.json`, oracle.carrier("package.json.txt"));
  oracle.write(`${prefix}jsconfig.json`, oracle.carrier("jsconfig.json.txt"));
  oracle.write(`${prefix}src/message.js`, oracle.carrier("message.js.txt"));
}

function noInputs(oracle: JsconfigOracle, name: string): string {
  return `Error: No supported source files were selected by TypeScript project \x60${path.join(oracle.root, name)}\x60; no files were checked. Check the project's files/include/exclude and Vize ignores.\n`;
}

function nativeNoInputs(oracle: JsconfigOracle, name: string): string {
  // Pinned native program.go validates checkJs against allowJs. The parser's
  // ForEachPropertyAssignment selects the first of those two authored keys:
  // this unchanged inverse places allowJs at line 3, column 5.
  return `error TS18003: No inputs were found in config file '${path.join(oracle.root, name)}'. Specified 'include' paths were '["src/**/*"]' and 'exclude' paths were '[]'.\n${name}(3,5): error TS5052: Option 'checkJs' cannot be specified without specifying option 'allowJs'.\n`;
}

void test("configless CLI discovers actual JavaScript project names and native defaults", async (t) => {
  await t.test(
    "default jsconfig omits allowJs and preserves JSDoc diagnostics through repair",
    () => {
      const oracle = new JsconfigOracle("implicit-default");
      let failure: unknown = null;
      try {
        prepare(oracle);
        const reference = oracle.stock("native-original", "jsconfig.json");
        const original = oracle.check("default-original");
        const repeated = oracle.check("default-repeated");
        oracle.write("src/message.js", oracle.carrier("message-repaired.js.txt"));
        const repairedReference = oracle.stock("native-repaired", "jsconfig.json");
        const repaired = oracle.check("default-repaired");
        stock(reference, "src/message.js", false);
        whole(original, 1, report(false));
        assert.equal(repeated.stdout, original.stdout);
        assert.equal(repeated.stderr, original.stderr);
        assert.equal(repeated.status, original.status);
        stock(repairedReference, "src/message.js", true);
        whole(repaired, 0, report(true));
      } catch (error) {
        failure = error;
        throw error;
      } finally {
        oracle.finish(failure);
      }
    },
  );

  await t.test(
    "same-directory tsconfig wins while explicit jsconfig retains its own program",
    () => {
      const oracle = new JsconfigOracle("same-directory-precedence");
      let failure: unknown = null;
      try {
        prepare(oracle);
        oracle.write("tsconfig.json", oracle.carrier("jsconfig-deny.json.txt"));
        const reference = oracle.stock("native-explicit-jsconfig", "jsconfig.json");
        const selected = oracle.check("explicit-jsconfig", ["--tsconfig", "jsconfig.json"]);
        const discovered = oracle.check("default-tsconfig");
        const denied = oracle.check("explicit-js-input-denied", ["src/message.js"]);
        const nativeDenied = oracle.stock("native-tsconfig-denied", "tsconfig.json", [
          "--listFilesOnly",
        ]);
        stock(reference, "src/message.js", false);
        whole(selected, 1, report(false));
        whole(discovered, 2, emptyReport, noInputs(oracle, "tsconfig.json"));
        whole(denied, 0, emptyReport);
        assert.equal(nativeDenied.status, 1);
        assert.equal(nativeDenied.stderr, "");
        assert.equal(nativeDenied.stdout, nativeNoInputs(oracle, "tsconfig.json"));
      } catch (error) {
        failure = error;
        throw error;
      } finally {
        oracle.finish(failure);
      }
    },
  );

  await t.test("a nearest package jsconfig owns default checks below an ancestor tsconfig", () => {
    const oracle = new JsconfigOracle("nearest-package");
    let failure: unknown = null;
    try {
      oracle.write("tsconfig.json", oracle.carrier("jsconfig-deny.json.txt"));
      prepare(oracle, "packages/app/");
      const reference = oracle.stock("native-package", "packages/app/jsconfig.json");
      const result = oracle.check("package-default", [], path.join(oracle.root, "packages/app"));
      stock(reference, "packages/app/src/message.js", false);
      whole(result, 1, report(false));
    } catch (error) {
      failure = error;
      throw error;
    } finally {
      oracle.finish(failure);
    }
  });

  await t.test(
    "explicit allowJs false remains a real exclusion and empty projects refuse success",
    () => {
      const oracle = new JsconfigOracle("explicit-false");
      let failure: unknown = null;
      try {
        prepare(oracle);
        oracle.write("jsconfig.json", oracle.carrier("jsconfig-deny.json.txt"));
        const native = oracle.stock("native-false", "jsconfig.json", ["--listFilesOnly"]);
        const defaultResult = oracle.check("default-false");
        const explicitResult = oracle.check("explicit-false", ["src/message.js"]);
        assert.equal(native.status, 1);
        assert.equal(native.stderr, "");
        assert.equal(native.stdout, nativeNoInputs(oracle, "jsconfig.json"));
        whole(defaultResult, 2, emptyReport, noInputs(oracle, "jsconfig.json"));
        whole(explicitResult, 0, emptyReport);
      } catch (error) {
        failure = error;
        throw error;
      } finally {
        oracle.finish(failure);
      }
    },
  );

  await t.test("unchecked JavaScript is admitted by jsconfig without inventing diagnostics", () => {
    const oracle = new JsconfigOracle("unchecked");
    let failure: unknown = null;
    try {
      prepare(oracle);
      oracle.write(
        "jsconfig.json",
        oracle.carrier("jsconfig.json.txt").replace('"checkJs": true', '"checkJs": false'),
      );
      const reference = oracle.stock("native-unchecked", "jsconfig.json");
      const result = oracle.check("default-unchecked");
      stock(reference, "src/message.js", true);
      whole(result, 0, report(true, { ...options, checkJs: false }));
    } catch (error) {
      failure = error;
      throw error;
    } finally {
      oracle.finish(failure);
    }
  });

  await t.test("each jsconfig's own filename defaults override inherited options", () => {
    const oracle = new JsconfigOracle("inherited-filename-defaults");
    let failure: unknown = null;
    try {
      prepare(oracle);
      oracle.write(
        "base.json",
        JSON.stringify({
          compilerOptions: {
            allowJs: false,
            maxNodeModuleJsDepth: 0,
            skipLibCheck: false,
            noEmit: false,
          },
        }),
      );
      const extend = (source: string): string =>
        source.replace('  "include"', '  "extends": "./base.json",\n  "include"');
      oracle.write("jsconfig.json", extend(oracle.carrier("jsconfig.json.txt")));
      const reference = oracle.stock("native-inherited-defaults", "jsconfig.json");
      const result = oracle.check("default-inherited-defaults");
      oracle.write("jsconfig.json", extend(oracle.carrier("jsconfig-deny.json.txt")));
      const denied = oracle.stock("native-inherited-explicit-false", "jsconfig.json", [
        "--listFilesOnly",
      ]);
      const excluded = oracle.check("default-inherited-explicit-false");
      stock(reference, "src/message.js", false);
      whole(result, 1, report(false));
      assert.equal(denied.status, 1);
      assert.equal(denied.stderr, "");
      assert.equal(denied.stdout, nativeNoInputs(oracle, "jsconfig.json"));
      whole(excluded, 2, emptyReport, noInputs(oracle, "jsconfig.json"));
    } catch (error) {
      failure = error;
      throw error;
    } finally {
      oracle.finish(failure);
    }
  });

  await t.test(
    "explicit workspace roots preserve both actual config names and complete programs",
    () => {
      const oracle = new JsconfigOracle("mixed-workspace");
      let failure: unknown = null;
      try {
        prepare(oracle, "packages/javascript/");
        const tsOptions = {
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
          strict: true,
          target: "ES2022",
          types: [],
        };
        oracle.write("package.json", oracle.carrier("package.json.txt"));
        oracle.write("tsconfig.json", JSON.stringify({ files: [] }));
        oracle.write(
          "packages/typescript/tsconfig.json",
          JSON.stringify({ compilerOptions: tsOptions, include: ["src/**/*"] }),
        );
        oracle.write("packages/typescript/src/value.ts", oracle.carrier("value.ts.txt"));
        const nativeJs = oracle.stock(
          "native-workspace-javascript",
          "packages/javascript/jsconfig.json",
        );
        const nativeTs = oracle.stock(
          "native-workspace-typescript",
          "packages/typescript/tsconfig.json",
        );
        const result = oracle.check("explicit-workspace", [
          "packages/javascript/src/message.js",
          "packages/typescript/src/value.ts",
        ]);
        stock(nativeJs, "packages/javascript/src/message.js", false);
        stock(nativeTs, "packages/typescript/src/value.ts", false, 1);
        whole(result, 1, {
          files: [
            {
              file: "packages/javascript/src/message.js",
              diagnostics: [`error:2:14 [TS2322] ${message}`],
            },
            {
              file: "packages/typescript/src/value.ts",
              diagnostics: [`error:1:14 [TS2322] ${message}`],
            },
          ],
          programs: [
            {
              root: "packages/javascript",
              tsconfig: "packages/javascript/jsconfig.json",
              compilerOptions: options,
              files: ["packages/javascript/src/message.js"],
            },
            {
              root: "packages/typescript",
              tsconfig: "packages/typescript/tsconfig.json",
              compilerOptions: tsOptions,
              files: ["packages/typescript/src/value.ts"],
            },
          ],
          errorCount: 2,
          warningCount: 0,
          fileCount: 2,
        });
      } catch (error) {
        failure = error;
        throw error;
      } finally {
        oracle.finish(failure);
      }
    },
  );
});
