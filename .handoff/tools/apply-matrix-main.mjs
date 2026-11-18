// Apply the matrix config (rule options + Vue 2 dialect) to the isolated main.ts.
import fs from "node:fs";
const file = `${process.argv[2]}/tools/support/fixtures/lint_divergence/main.ts`;
let s = fs.readFileSync(file, "utf8");
const rep = (a, b) => {
  if (!s.includes(a)) throw new Error(`missing: ${a.slice(0, 70)}`);
  s = s.replace(a, b);
};
rep(
  `import { isolationRoot, relocatePaths, withIsolatedFixture } from "./isolation.ts";\n`,
  `import { isolationRoot, relocatePaths, withIsolatedFixture } from "./isolation.ts";
import {
  matrixRuleOptions,
  matrixVueVersion,
  writeMatrixConfig,
  type MatrixRuleOption,
} from "./matrix_rule_options.ts";
import { vue2DialectDisabledRules } from "./vue2_dialect.ts";
`,
);
rep(
  `  project: Json,
): JsonObject | null {`,
  `  project: Json,
  matrix: { overrides: readonly MatrixRuleOption[]; ledger: Json } = {
    overrides: [],
    ledger: null,
  },
): JsonObject | null {`,
);
rep(
  `      project,
      files,
    ),
  );
}`,
  `      project,
      files,
      matrix,
    ),
  );
}`,
);
rep(
  `  project: Json,
  files: string[],
): JsonObject {`,
  `  project: Json,
  files: string[],
  matrix: { overrides: readonly MatrixRuleOption[]; ledger: Json },
): JsonObject {`,
);
rep(
  `  const patina =
    files.length === 0
      ? { results: [], findings: [] }
      : runPatina(cwd, project, args, launch, files);`,
  `  const vueVersion = matrixVueVersion(matrix.ledger, projectString(project, "fixturePath"));
  const config = writeMatrixConfig(matrix.overrides, vueVersion);
  const patina = (() => {
    try {
      return files.length === 0
        ? { results: [], findings: [] }
        : runPatina(cwd, project, args, launch, files, config);
    } finally {
      if (config !== null) fs.rmSync(path.dirname(config), { recursive: true, force: true });
    }
  })();`,
);
rep(
  `        rules: baselineRules,
        files,
      },
    );`,
  `        rules: baselineRules,
        files,
      },
      vueVersion === null ? new Set() : vue2DialectDisabledRules(root),
    );`,
);
rep(
  `  const launch = resolveVizeLaunch(root, args.vizeBin),`,
  `  const matrix = {
    overrides: matrixRuleOptions(root, ruleMap),
    ledger: readJson(path.join(root, "tests/_fixtures/fixture-compatibility-ledger.json")),
  };
  const launch = resolveVizeLaunch(root, args.vizeBin),`,
);
rep(
  `    const artifact = measureProject(root, args, launch, evidence, ruleMap, comparable, project);`,
  `    const artifact = measureProject(
      root,
      args,
      launch,
      evidence,
      ruleMap,
      comparable,
      project,
      matrix,
    );`,
);
fs.writeFileSync(file, s);
