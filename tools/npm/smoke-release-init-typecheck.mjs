import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

function runResult(command, args, options = {}) {
  const isWindowsBatch = process.platform === "win32" && /\.(cmd|bat)$/i.test(command);
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    encoding: "utf8",
    env: options.env ?? process.env,
    input: options.input,
    stdio: ["pipe", "pipe", "pipe"],
    shell: isWindowsBatch,
  });
  if (result.error != null) throw result.error;
  return result;
}

function run(command, args, options = {}) {
  const result = runResult(command, args, options);
  if (result.status !== 0) {
    const rendered = [result.stdout, result.stderr].filter(Boolean).join("\n").trim();
    throw new Error(
      [`${command} ${args.join(" ")} failed with exit ${result.status}`, rendered]
        .filter(Boolean)
        .join("\n"),
    );
  }
  return result.stdout;
}
function writeProjectFile(projectDir, filename, source) {
  const target = path.join(projectDir, filename);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, source);
}

export function isPathInsideOrEqual(parent, candidate, pathApi = path) {
  const relative = pathApi.relative(parent, candidate);
  if (relative === "") return true;
  // Compare ".." as a path segment so names such as "..cache" stay inside.
  if (relative === ".." || relative.startsWith(`..${pathApi.sep}`)) return false;
  // On Windows, paths on different drives produce an absolute relative path.
  return !pathApi.isAbsolute(relative);
}

function initTypecheckAppSource(language, failure = null) {
  const scriptLanguage = language === "typescript" ? ' lang="ts"' : "";
  const countDeclaration =
    language === "typescript"
      ? failure === "SFC script"
        ? 'const count: number = "wrong";'
        : "const count: number = 1;"
      : failure === "SFC script"
        ? '/** @type {number} */\nconst count = "wrong";'
        : "/** @type {number} */\nconst count = 1;";
  const labelDeclaration =
    failure === "SFC template" ? "const label = 1;" : 'const label = "ready";';
  const component =
    failure === "component prop" ? '<Counter count="wrong" />' : '<Counter :count="count" />';

  return [
    "<template>",
    `  ${component}`,
    "  <p>{{ label.toUpperCase() }}</p>",
    "</template>",
    "",
    `<script setup${scriptLanguage}>`,
    'import Counter from "./Counter.vue";',
    "",
    countDeclaration,
    labelDeclaration,
    "</script>",
    "",
  ].join("\n");
}

function initTypecheckCounterSource(language) {
  if (language === "typescript") {
    return [
      "<template>",
      "  <span>{{ count.toFixed(0) }}</span>",
      "</template>",
      "",
      '<script setup lang="ts">',
      "defineProps<{ count: number }>();",
      "</script>",
      "",
    ].join("\n");
  }

  return [
    "<template>",
    "  <span>{{ count.toFixed(0) }}</span>",
    "</template>",
    "",
    "<script setup>",
    "defineProps({ count: { type: Number, required: true } });",
    "</script>",
    "",
  ].join("\n");
}

function initTypecheckStandaloneSource(language, broken) {
  if (language === "typescript") {
    return broken ? 'export const count: number = "wrong";\n' : "export const count: number = 1;\n";
  }
  return broken
    ? '/** @type {number} */\nexport const count = "wrong";\n'
    : "/** @type {number} */\nexport const count = 1;\n";
}

function initTypecheckConsumerSource(broken) {
  const value = broken ? '"wrong"' : "1";
  return [
    'import Counter from "./Counter.vue";',
    "",
    `export const view = <Counter count={${value}} />;`,
    "",
  ].join("\n");
}

function createInitTypecheckProject(installDir, language, runtimePeerDependencies) {
  const projectDir = path.join(installDir, `init-smoke-${language}`);
  const sourceExtension = language === "typescript" ? "ts" : "js";
  const consumerExtension = language === "typescript" ? "tsx" : "jsx";
  writeProjectFile(
    projectDir,
    "package.json",
    `${JSON.stringify(
      {
        name: `vize-init-${language}-smoke`,
        private: true,
        type: "module",
        devDependencies:
          language === "typescript"
            ? {
                typescript: runtimePeerDependencies.typescript,
                vue: runtimePeerDependencies.vue,
              }
            : { vue: runtimePeerDependencies.vue },
      },
      null,
      2,
    )}\n`,
  );
  writeProjectFile(
    projectDir,
    `src/model.${sourceExtension}`,
    initTypecheckStandaloneSource(language, false),
  );
  writeProjectFile(projectDir, "src/Counter.vue", initTypecheckCounterSource(language));
  writeProjectFile(projectDir, "src/App.vue", initTypecheckAppSource(language));
  writeProjectFile(
    projectDir,
    `src/Consumer.${consumerExtension}`,
    initTypecheckConsumerSource(false),
  );
  return { consumerExtension, projectDir, sourceExtension };
}

function runGeneratedTypecheck(installDir, projectDir) {
  const binDir = path.join(installDir, "node_modules", ".bin");
  const inheritedPath = process.env.PATH ?? "";
  return runResult(
    process.env.NPM_BIN || "npm",
    ["run", "--silent", "vize:check", "--", "--format", "json", "--quiet"],
    {
      cwd: projectDir,
      env: { ...process.env, PATH: `${binDir}${path.delimiter}${inheritedPath}` },
    },
  );
}

function checkOutput(result) {
  return [result.stdout, result.stderr].filter(Boolean).join("\n").trim();
}

function assertGeneratedTypecheckPass(installDir, projectDir, phase) {
  const result = runGeneratedTypecheck(installDir, projectDir);
  const output = checkOutput(result);
  assert.equal(result.status, 0, `${phase} should pass\n${output}`);
  const report = JSON.parse(result.stdout);
  assert.equal(report.errorCount, 0, `${phase} reported errors\n${output}`);
}

function assertGeneratedTypecheckFailure(installDir, projectDir, phase, filename, code) {
  const result = runGeneratedTypecheck(installDir, projectDir);
  const output = checkOutput(result);
  assert.notEqual(result.status, 0, `${phase} unexpectedly passed`);
  assert.ok(
    output.includes(filename) || output.includes(path.basename(filename)),
    `${phase} lost authored filename\n${output}`,
  );
  assert.match(output, new RegExp(`TS${code}\\b`), `${phase} lost diagnostic TS${code}`);
}

function runInitTypecheckCase(installDir, vizeBin, language, runtimePeerDependencies) {
  const { consumerExtension, projectDir, sourceExtension } = createInitTypecheckProject(
    installDir,
    language,
    runtimePeerDependencies,
  );
  const initArgs = [
    vizeBin,
    "init",
    projectDir,
    "--yes",
    "--no-lint",
    "--no-bundler",
    "--no-fmt",
    "--typecheck",
    "--no-editor",
    "--no-install",
  ];
  run(process.execPath, initArgs, { cwd: installDir });

  const packagePath = path.join(projectDir, "package.json");
  const tsconfigPath = path.join(projectDir, "tsconfig.json");
  const configPath = path.join(projectDir, "vize.config.ts");
  const manifest = JSON.parse(fs.readFileSync(packagePath, "utf8"));
  const tsconfig = JSON.parse(fs.readFileSync(tsconfigPath, "utf8"));
  assert.equal(manifest.scripts["vize:check"], "vize check");
  assert.equal(tsconfig.compilerOptions.allowJs, language === "javascript" ? true : undefined);
  assert.equal(tsconfig.compilerOptions.checkJs, language === "javascript" ? true : undefined);

  const initializedBytes = [packagePath, tsconfigPath, configPath].map((filename) =>
    fs.readFileSync(filename, "utf8"),
  );
  run(process.execPath, initArgs, { cwd: installDir });
  assert.deepEqual(
    [packagePath, tsconfigPath, configPath].map((filename) => fs.readFileSync(filename, "utf8")),
    initializedBytes,
    `${language} init should be idempotent`,
  );

  assertGeneratedTypecheckPass(installDir, projectDir, `${language} initial clean project`);

  const standaloneFile = `src/model.${sourceExtension}`;
  writeProjectFile(projectDir, standaloneFile, initTypecheckStandaloneSource(language, true));
  assertGeneratedTypecheckFailure(
    installDir,
    projectDir,
    `${language} standalone source`,
    standaloneFile,
    2322,
  );
  writeProjectFile(projectDir, standaloneFile, initTypecheckStandaloneSource(language, false));
  assertGeneratedTypecheckPass(installDir, projectDir, `${language} repaired standalone source`);

  for (const [failure, code] of [
    ["SFC script", 2322],
    ["SFC template", 2339],
    ["component prop", 2322],
  ]) {
    writeProjectFile(projectDir, "src/App.vue", initTypecheckAppSource(language, failure));
    assertGeneratedTypecheckFailure(
      installDir,
      projectDir,
      `${language} ${failure}`,
      "src/App.vue",
      code,
    );
    writeProjectFile(projectDir, "src/App.vue", initTypecheckAppSource(language));
    assertGeneratedTypecheckPass(installDir, projectDir, `${language} repaired ${failure}`);
  }

  const consumerFile = `src/Consumer.${consumerExtension}`;
  writeProjectFile(projectDir, consumerFile, initTypecheckConsumerSource(true));
  assertGeneratedTypecheckFailure(
    installDir,
    projectDir,
    `${language} JSX consumer`,
    consumerFile,
    2322,
  );
  writeProjectFile(projectDir, consumerFile, initTypecheckConsumerSource(false));
  assertGeneratedTypecheckPass(installDir, projectDir, `${language} repaired JSX consumer`);
}

export function runInitTypecheckChecks(
  installDir,
  vizeBin,
  repositoryRoot,
  runtimePeerDependencies,
) {
  const installedVizeRoot = fs.realpathSync(path.dirname(path.dirname(vizeBin)));
  assert.ok(
    !isPathInsideOrEqual(repositoryRoot, installedVizeRoot),
    `runtime resolved repository-local vize package: ${installedVizeRoot}`,
  );
  for (const language of ["typescript", "javascript"]) {
    runInitTypecheckCase(installDir, vizeBin, language, runtimePeerDependencies);
  }
  console.log("runtime: packed vize init typecheck clean/broken/repaired matrix");
}
