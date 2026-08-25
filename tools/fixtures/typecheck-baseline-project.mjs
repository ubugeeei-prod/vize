import { existsSync, mkdirSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";

import {
  mergePathRewrites,
  rewriteOutsideAliasPaths,
} from "./typecheck-baseline-outside-aliases.mjs";
import {
  isolatedOverlayBaseUrl,
  rewriteOutsidePackagePaths,
  rewriteOutsideRootDirs,
  rewriteOutsideTypeRoots,
} from "./typecheck-baseline-outside-paths.mjs";
import { rewriteOutsideVueCompilerOptions } from "./typecheck-baseline-outside-vue-compiler.mjs";
import { rewriteOutsideCompilerPlugins } from "./typecheck-baseline-outside-compiler-plugins.mjs";
import {
  mergeLocalVueRuntimePaths,
  rewriteLocalVueRuntimePaths,
} from "./typecheck-baseline-outside-vue-runtime-paths.mjs";
import { typecheckCorpusGlobs } from "./tool-matrix-command.mjs";

/**
 * Materialize the exact Vue corpus Vize checked as a vue-tsc project.
 *
 * The fixture's pinned tsconfig remains the source of compiler options, while
 * `files` replaces solution-style or incomplete include lists. This makes the
 * baseline compare the same authored SFCs instead of silently checking none.
 *
 * `files` alone is not enough, though, and #3738 is what that costs. A `files`
 * list seeds the program with those roots and nothing else, so every ambient
 * declaration the fixture owns — `declare global`, `declare namespace`, module
 * augmentation — falls out of the program unless some SFC happens to import it,
 * which ambient files by definition are not imported. The baseline then reports
 * the project's own globals as undeclared and the ledger scores that against
 * Vize: on run 30738583070 all 24 of lx-music-desktop's "false negatives" were
 * `Cannot find namespace 'LX'` or `Property 'lx' does not exist on 'Window'`,
 * both declared in its `src/**\/*.d.ts`, and restoring the glob below takes the
 * baseline from 25 diagnostics over the Vue corpus to 1 — the one Vize agrees
 * with. `elk`, `voicevox`, `vuestic-admin` and `misskey` fail the same way.
 *
 * Declaration files are the right thing to add back and the only thing:
 * they are the fixture's type *environment*, never the comparison's subject, so
 * they cannot change which SFCs are checked, and every diagnostic reported
 * inside one is already excluded from scoring for being non-Vue. `exclude` is
 * ours rather than the fixture's for the same reason — the glob is ours, and it
 * must not reach into installed or built output. It cannot narrow the compared
 * corpus, because `exclude` never applies to `files`.
 *
 * The generated baseline lives beside the source config rather than at fixture
 * root. Several workspace fixtures, including vue-vben-admin, resolve
 * `compilerOptions.types` through package-local `node_modules` entries such as
 * `playground/node_modules/@vben/types`; moving the config to a root-level
 * `.vize-baseline` directory makes TypeScript skip those package-local links and
 * turns a usable baseline into a configuration failure.
 *
 * The baseline also has to list non-Vue authored scripts that the compared SFCs
 * import. Composite projects reject imported files that are not in the project
 * file list (`TS6307`), and that is an instrument failure rather than a Vize
 * diagnostic. These support globs deliberately exclude `.vue`, so `files` still
 * fixes the comparable SFC corpus while TypeScript can build the same project
 * graph around it.
 *
 * Dot-directories need literal include roots, because a TypeScript wildcard
 * segment never descends into one: `<fixture>/**\/*.d.ts` misses
 * `.nuxt/imports.d.ts` and `docs/.vitepress/utils.ts`, while literal
 * `<fixture>/.nuxt/**` or `<fixture>/docs/.vitepress/**` matches them. The
 * source config's own directory and every checked-file dot ancestor are therefore
 * globbed by name.
 */
export function materializeBaselineProject(fixtureRoot, reportDir, project, vizeReport) {
  const sourceProject = project.typecheckPerformance?.baseline?.tsconfig ?? project.tsconfig;
  const sourcePath = resolve(fixtureRoot, sourceProject);
  const artifactPath = join(reportDir, `${project.id}-vue-tsc.tsconfig.json`);
  const outputPath = join(
    dirname(sourcePath),
    ".vize-baseline",
    `${project.id}-vue-tsc.tsconfig.json`,
  );
  const configDir = dirname(outputPath);
  const globRoots = vueGlobSourceRoots(fixtureRoot, project);
  // `readdirSync` order is filesystem-dependent and `Set` keeps insertion order,
  // so the discovered roots are sorted to keep the generated config byte-stable
  // for the same fixture.
  const dotRoots = [
    ...new Set([
      ...dotDirectoryIncludeRoots(fixtureRoot, vizeReport),
      ...discoverDotDirectoryIncludeRoots(globRoots),
    ]),
  ].sort((a, b) => a.localeCompare(b));
  const ambientRoots = [
    ...new Set(
      [fixtureRoot, dirname(sourcePath), ...dotRoots].map((root) =>
        configRelativePath(configDir, root),
      ),
    ),
  ];
  const sourceRoots = [
    ...new Set([...globRoots, ...dotRoots].map((root) => configRelativePath(configDir, root))),
  ];
  const config = {
    extends: configRelativePath(configDir, sourcePath),
    compilerOptions: {
      // The release matrix runs the current pinned vue-tsc/TypeScript baseline
      // against old fixture configs. Inherited TS 6 deprecation errors are not
      // project diagnostics, and without this the baseline aborts before it can
      // measure the same SFC corpus as Vize.
      ignoreDeprecations: "6.0",
      // The generated config is the measurement harness, not a source root. If
      // TypeScript infers `rootDir` from `.vize-baseline`, every file outside
      // that directory becomes TS6059 noise and the baseline stops measuring
      // Vize. Pin it to the fixture corpus root instead.
      rootDir: configRelativePath(configDir, fixtureRoot),
      ...outsideCompilerOptions(fixtureRoot, sourcePath, configDir),
    },
    files: vizeReport.files
      .slice(0, vizeReport.fileCount)
      .map((entry) => configRelativePath(configDir, resolve(fixtureRoot, entry.file))),
    include: [
      ...ambientRoots.map((root) => `${root}/**/*.d.ts`),
      ...sourceRoots.flatMap((root) => sourceIncludeGlobs(root)),
      ...dotRoots.map((root) => `${configRelativePath(configDir, root)}/**/*.vue`),
    ],
    exclude: ambientRoots.flatMap((root) => [`${root}/**/node_modules/**`, `${root}/**/dist/**`]),
    references: [],
  };
  const vueCompilerOptions = rewriteOutsideVueCompilerOptions(fixtureRoot, sourcePath, configDir);
  if (vueCompilerOptions != null) config.vueCompilerOptions = vueCompilerOptions;
  const source = `${JSON.stringify(config, null, 2)}\n`;
  mkdirSync(configDir, { recursive: true });
  writeFileSync(outputPath, source);
  writeFileSync(artifactPath, source);
  return { path: outputPath, source, sourceProject };
}

function outsideCompilerOptions(fixtureRoot, sourcePath, configDir) {
  const options = {};
  const paths = mergeLocalVueRuntimePaths(
    mergePathRewrites(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, configDir),
      rewriteOutsideAliasPaths(fixtureRoot, sourcePath, configDir),
    ),
    rewriteLocalVueRuntimePaths(fixtureRoot, configDir),
  );
  if (paths != null) options.paths = paths;
  if (paths != null && isolatedOverlayBaseUrl(sourcePath, fixtureRoot) != null) {
    options.baseUrl = ".";
  }
  const typeRoots = rewriteOutsideTypeRoots(fixtureRoot, sourcePath, configDir);
  if (typeRoots != null) options.typeRoots = typeRoots;
  const rootDirs = rewriteOutsideRootDirs(fixtureRoot, sourcePath, configDir);
  if (rootDirs != null) options.rootDirs = rootDirs;
  const plugins = rewriteOutsideCompilerPlugins(fixtureRoot, sourcePath, configDir);
  if (plugins != null) options.plugins = plugins;
  return options;
}

function configRelativePath(from, to) {
  const path = relative(from, to).replaceAll("\\", "/");
  if (isAbsolute(path) || /^[A-Za-z]:\//u.test(path)) return path;
  return path.startsWith(".") ? path : `./${path}`;
}

function vueGlobSourceRoots(fixtureRoot, project) {
  const globs = typecheckCorpusGlobs(project);
  if (!Array.isArray(globs)) return [];
  return globs.map((glob) => resolve(fixtureRoot, literalGlobRoot(glob)));
}

function literalGlobRoot(glob) {
  if (typeof glob !== "string" || glob.length === 0) return ".";
  const firstMagic = glob.search(/[*?[\]{}]/u);
  const prefix = firstMagic < 0 ? glob : glob.slice(0, firstMagic);
  const slash = prefix.lastIndexOf("/");
  const root = slash < 0 ? "" : prefix.slice(0, slash);
  return root.length === 0 ? "." : root;
}

function dotDirectoryIncludeRoots(fixtureRoot, vizeReport) {
  const roots = [];
  const files = Array.isArray(vizeReport.files)
    ? vizeReport.files.slice(0, vizeReport.fileCount)
    : [];
  for (const entry of files) {
    if (typeof entry?.file !== "string") continue;
    const segments = entry.file.replaceAll("\\", "/").split("/");
    for (let index = 0; index < segments.length - 1; index += 1) {
      const segment = segments[index];
      if (!segment.startsWith(".") || segment === "." || segment === "..") continue;
      roots.push(resolve(fixtureRoot, ...segments.slice(0, index + 1)));
    }
  }
  return roots;
}

function discoverDotDirectoryIncludeRoots(sourceRoots) {
  const roots = [];
  for (const sourceRoot of sourceRoots) collectDotDirectories(sourceRoot, roots);
  return roots;
}

function collectDotDirectories(directory, roots) {
  if (!existsSync(directory)) return;
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    if (ignoredDirectory(entry.name)) continue;
    const child = resolve(directory, entry.name);
    if (isDotDirectory(entry.name)) roots.push(child);
    collectDotDirectories(child, roots);
  }
}

function ignoredDirectory(name) {
  return name === "node_modules" || name === "dist" || name === ".git" || name === ".vize-baseline";
}

function isDotDirectory(name) {
  return name.startsWith(".") && name !== "." && name !== "..";
}

function sourceIncludeGlobs(root) {
  return [
    `${root}/**/*.ts`,
    `${root}/**/*.tsx`,
    `${root}/**/*.mts`,
    `${root}/**/*.cts`,
    `${root}/**/*.js`,
    `${root}/**/*.jsx`,
    `${root}/**/*.mjs`,
    `${root}/**/*.cjs`,
    `${root}/**/*.json`,
  ];
}
