import { appendFileSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { changedPaths, planSourceChecks } from "./plan-source-checks.ts";
import { nativeSetupCaptureRequired } from "./native-setup-capture.mjs";
import { nativeSsrCaptureRequired } from "./native-ssr-capture.mjs";
import { nativeScopedCaptureRequired } from "./native-scoped-capture.mjs";
import { nativeVaporCaptureRequired } from "./native-vapor-capture.mjs";
import { nativeAttributeValues7502CaptureRequired } from "./native-attribute-values-7502-capture.mjs";

export function planNativeRuntime(
  paths: readonly string[],
  rust?: { packages: readonly string[] },
) {
  const source = planSourceChecks(paths);
  // Source and test changes in either native producer (including transitive
  // Cargo dependencies such as L0 derive) qualify all of its runtime outputs.
  const nativeProducer =
    rust?.packages.some((pkg) => pkg === "vize_l4" || pkg === "vize_atelier_sfc") ?? false;
  const broad =
    paths.length === 0 ||
    nativeProducer ||
    (source.rust && source.tooling && source.playground) ||
    paths.some((path) =>
      /^(?:Cargo\.(?:toml|lock)|rust-toolchain\.toml|package\.json|pnpm-(?:lock|workspace)\.yaml|\.cargo\/|\.github\/(?:workflows\/(?:pr-source-checks|pr-rust-checks|check)\.yml|actions\/setup-)|tools\/support\/compat\/github\/plan-native-runtime\.ts)/.test(
        path,
      ),
    );
  const dom = nativeSetupCaptureRequired(paths);
  const typechecker =
    rust?.packages.includes("vize_canon") ||
    paths.some((path) =>
      /^(?:crates\/vize_canon\/|\.github\/actions\/test-native-canon\/)/.test(path),
    );
  const lsp =
    rust?.packages.some((pkg) => pkg === "vize" || pkg === "vize_maestro") ||
    paths.some((path) =>
      /^(?:crates\/vize(?:_maestro)?\/|tests\/tooling\/(?:lsp-|support\/(?:lsp\/|editor-jsconfig|package-private-imports|project-navigation|typecheck-dependency))|tests\/differential\/|tests\/_fixtures\/differential\/lsp\/|\.github\/actions\/test-native-lsp\/)/.test(
        path,
      ),
    );
  const lanes = [
    ["canon", typechecker],
    ["lsp", lsp],
    ["ssr", nativeSsrCaptureRequired(paths) || nativeScopedCaptureRequired(paths)],
    [
      "scoped-css",
      nativeSsrCaptureRequired(paths) ||
        nativeScopedCaptureRequired(paths) ||
        paths.some((path) =>
          /^(?:crates\/vize_atelier_sfc\/src\/native\/(?:scoped|class_list)|crates\/vize_atelier_ssr\/|tests\/tooling\/(?:native-sfc-scoped|ssr-fragment-css-vars))/.test(
            path,
          ),
        ),
    ],
    ["setup", dom],
    ["vapor", nativeVaporCaptureRequired(paths)],
    ["attributes", nativeAttributeValues7502CaptureRequired(paths)],
    [
      "selected-template",
      dom ||
        paths.some((path) =>
          /^(?:davinci\/vize_l[0-4](?:_to_l[0-4])?\/src\/|tools\/support\/compat\/davinci\/selected-template|\.github\/actions\/test-native-selected-template)/.test(
            path,
          ),
        ),
    ],
  ] as const;
  const selected = lanes.filter(([, required]) => broad || required).map(([lane]) => lane);
  return { run: selected.length > 0, matrix: { lane: selected.length ? selected : ["none"] } };
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [base, head, rustFile] = process.argv.slice(2);
  if (![base, head].every((sha) => /^[0-9a-f]{40}$/.test(sha ?? "")))
    throw new Error("expected full base and head commit SHAs");
  const rust = rustFile ? JSON.parse(readFileSync(rustFile, "utf8")) : undefined;
  const plan = planNativeRuntime(/^0+$/.test(base) ? [] : changedPaths(base, head), rust);
  if (process.env.GITHUB_OUTPUT)
    appendFileSync(
      process.env.GITHUB_OUTPUT,
      `native-runtime=${plan.run}\nnative-runtime-matrix=${JSON.stringify(plan.matrix)}\n`,
    );
  process.stdout.write(`${JSON.stringify(plan)}\n`);
}
