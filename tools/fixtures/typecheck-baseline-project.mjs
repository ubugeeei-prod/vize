import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";

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
 * The source config's own directory is also globbed as well as the fixture root,
 * because a TypeScript wildcard segment never descends into a dot-directory:
 * `<fixture>/**\/*.d.ts` misses `.nuxt/imports.d.ts` while `<fixture>/.nuxt/**`
 * matches it, the segment being literal. That is the whole of elk's generated
 * type environment, and every project whose baseline config is generated into a
 * dot-directory has the same shape. For the rest the second root is already
 * inside the first and adds nothing.
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
  const ambientRoots = [
    ...new Set(
      [fixtureRoot, dirname(sourcePath)].map((root) => configRelativePath(configDir, root)),
    ),
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
    },
    files: vizeReport.files
      .slice(0, vizeReport.fileCount)
      .map((entry) => configRelativePath(configDir, resolve(fixtureRoot, entry.file))),
    include: ambientRoots.map((root) => `${root}/**/*.d.ts`),
    exclude: ambientRoots.flatMap((root) => [`${root}/**/node_modules/**`, `${root}/**/dist/**`]),
    references: [],
  };
  const source = `${JSON.stringify(config, null, 2)}\n`;
  mkdirSync(configDir, { recursive: true });
  writeFileSync(outputPath, source);
  writeFileSync(artifactPath, source);
  return { path: outputPath, source, sourceProject };
}

function configRelativePath(from, to) {
  const path = relative(from, to).replaceAll("\\", "/");
  if (isAbsolute(path) || /^[A-Za-z]:\//u.test(path)) return path;
  return path.startsWith(".") ? path : `./${path}`;
}
