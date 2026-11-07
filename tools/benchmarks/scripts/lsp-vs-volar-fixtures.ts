/**
 * Projects for `lsp-vs-volar.ts`: real ones at the registry's pinned
 * revisions and generated ones (lsp-vs-volar-scale.ts), and the authored spots
 * the scripted session edits and queries in each.
 *
 * Every anchor is unique text in the pinned file; a missing or repeated anchor
 * throws instead of measuring a guessed position. Expectations are what a
 * correct answer must contain, so a fast empty or wrong answer counts as a
 * failure rather than a win.
 */

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve, sep } from "node:path";

import { SCALE_FIXTURES } from "./lsp-vs-volar-scale-fixtures.ts";
import { type Anchor, anchorIn } from "./lsp-vs-volar-text.ts";

export type RequestSpec =
  | { kind: "hover"; name: string; anchor: Anchor; expect: string[] }
  | { kind: "completion"; name: string; anchor: Anchor; expect: string[] }
  | { kind: "definition"; name: string; anchor: Anchor; expectPath: string }
  | { kind: "references"; name: string; anchor: Anchor; min: number }
  | { kind: "rename"; name: string; anchor: Anchor; newName: string; min: number }
  | { kind: "semanticTokens"; name: string; min: number };

export type RequestKind = RequestSpec["kind"];

export const REQUEST_KINDS: RequestKind[] = [
  "hover",
  "completion",
  "definition",
  "references",
  "rename",
  "semanticTokens",
];

export interface FixtureSpec {
  id: string;
  /** SFC edited and queried, relative to the project root (the opened workspace). */
  file: string;
  /** Projects whose programs hold `.vue`; the content-mapper editor declares the mapper in each. */
  tsconfigs: string[];
  /**
   * A registry clone with commands run once after install, or files generated
   * from the workspace catalog's Vue version and installed with npm.
   */
  source:
    | { kind: "registry"; prepare?: string[] }
    | { kind: "generated"; files: (vue: string) => Map<string, string> };
  /** Fixtures on one curve differ only in unrelated project size, `sfcs` SFCs each. */
  curve?: { name: string; sfcs: number };
  /** The script probe goes right after this text, the template probe after the other. */
  scriptProbeAfter: string;
  templateProbeAfter: string;
  /** Indentation of the template probe line. */
  templateIndent: string;
  requests: RequestSpec[];
}

/** Planted at open: the cold measure waits for this error, the first edit repairs it. */
export const SCRIPT_BROKEN = "const benchProbe: number = 'bench';";
export const SCRIPT_CLEAN = "const benchProbe: number = 1;";
export const TEMPLATE_CLEAN = "benchProbe.toFixed(2)";
export const TEMPLATE_BROKEN = "benchProbe.toFixed('bench')";
/** TS2322 for the script probe, TS2345 for the template probe. */
export const SCRIPT_CODE = "2322";
export const TEMPLATE_CODE = "2345";

export const FIXTURES: FixtureSpec[] = [
  {
    id: "misskey",
    file: "packages/frontend/src/components/MkNote.vue",
    tsconfigs: ["packages/frontend/tsconfig.json"],
    source: { kind: "registry" },
    scriptProbeAfter: '<script lang="ts" setup>\n',
    templateProbeAfter:
      '\t<MkNoteSub v-if="appearNote.replyId && !renoteCollapsed" :note="appearNote?.reply ?? null" :class="$style.replyTo"/>\n',
    templateIndent: "\t",
    requests: [
      {
        kind: "hover",
        name: "script binding",
        anchor: anchorIn("const appearNote = getAppearNote(note) ?? note;", "appearNote"),
        expect: ["appearNote:"],
      },
      {
        kind: "hover",
        name: "template expression",
        anchor: anchorIn("{{ appearNote.channel.name }}", "appearNote"),
        expect: ["appearNote:"],
      },
      {
        kind: "hover",
        name: "component tag",
        anchor: anchorIn('<MkNoteHeader :note="appearNote" :mini="true"/>', "MkNoteHeader"),
        expect: ["MkNoteHeader"],
      },
      {
        kind: "hover",
        name: "component prop",
        anchor: anchorIn('<MkNoteHeader :note="appearNote" :mini="true"/>', "note="),
        expect: ["note"],
      },
      {
        kind: "completion",
        name: "script member",
        anchor: anchorIn("computed(() => prefer.s.showSoftWordMutedWord)", "showSoftWordMutedWord"),
        expect: ["showSoftWordMutedWord"],
      },
      {
        kind: "completion",
        name: "template member",
        anchor: anchorIn("{{ appearNote.channel.name }}", "channel"),
        expect: ["channel", "user"],
      },
      {
        kind: "definition",
        name: "component tag",
        anchor: anchorIn('<MkNoteHeader :note="appearNote" :mini="true"/>', "MkNoteHeader"),
        expectPath: "src/components/MkNoteHeader.vue",
      },
      {
        kind: "definition",
        name: "imported function",
        anchor: anchorIn("const appearNote = getAppearNote(note) ?? note;", "getAppearNote"),
        expectPath: "src/utility/get-appear-note.ts",
      },
      {
        kind: "references",
        name: "local binding",
        anchor: anchorIn("const renoteCollapsed = ref(", "renoteCollapsed"),
        min: 12,
      },
      {
        kind: "references",
        name: "imported function",
        anchor: anchorIn(
          "import { getAppearNote } from '@/utility/get-appear-note.js';",
          "getAppearNote",
        ),
        min: 3,
      },
      {
        kind: "rename",
        name: "local binding",
        anchor: anchorIn("const renoteCollapsed = ref(", "renoteCollapsed"),
        newName: "renoteFolded",
        min: 12,
      },
      { kind: "semanticTokens", name: "document", min: 1 },
    ],
  },
  {
    id: "n8n",
    file: "packages/frontend/editor-ui/src/features/ndv/parameters/components/ResourceLocator/ResourceLocator.vue",
    tsconfigs: ["packages/frontend/editor-ui/tsconfig.json"],
    source: {
      kind: "registry",
      // n8n's own recipe for its editor's workspace dependencies without full type builds.
      prepare: ["exec", "turbo", "run", "build:unchecked", "--filter=n8n-editor-ui^..."],
    },
    scriptProbeAfter: '<script setup lang="ts">\n',
    templateProbeAfter: '\t\t:data-test-id="`resource-locator-${parameter.name}`"\n\t>\n',
    templateIndent: "\t\t",
    requests: [
      {
        kind: "hover",
        name: "script binding",
        anchor: anchorIn("const currentQueryResults = computed<", "currentQueryResults"),
        expect: ["currentQueryResults:"],
      },
      {
        kind: "hover",
        name: "template expression",
        anchor: anchorIn(':resources="currentQueryResults"', "currentQueryResults"),
        expect: ["currentQueryResults:"],
      },
      {
        kind: "hover",
        name: "component tag",
        anchor: anchorIn("<ResourceLocatorDropdown\n", "ResourceLocatorDropdown"),
        expect: ["ResourceLocatorDropdown"],
      },
      {
        kind: "hover",
        name: "component prop",
        anchor: anchorIn(':resources="currentQueryResults"', "resources"),
        expect: ["resources"],
      },
      {
        kind: "completion",
        name: "script member",
        anchor: anchorIn("if (currentMode.value.placeholder) {", "placeholder"),
        expect: ["placeholder"],
      },
      {
        kind: "completion",
        name: "template member",
        anchor: anchorIn("{{ currentResponse.errorDetails.httpCode }}", "errorDetails"),
        expect: ["errorDetails"],
      },
      {
        kind: "definition",
        name: "component tag",
        anchor: anchorIn("<ResourceLocatorDropdown\n", "ResourceLocatorDropdown"),
        expectPath: "ResourceLocator/ResourceLocatorDropdown.vue",
      },
      {
        kind: "definition",
        name: "imported function",
        anchor: anchorIn("const i18n = useI18n();", "useI18n"),
        expectPath: "@n8n/i18n/src/index.ts",
      },
      {
        kind: "references",
        name: "local binding",
        anchor: anchorIn("const isListMode = computed(", "isListMode"),
        min: 12,
      },
      {
        kind: "references",
        name: "imported function",
        anchor: anchorIn(
          "import { ndvEventBus } from '@/features/ndv/shared/ndv.eventBus';",
          "ndvEventBus",
        ),
        min: 3,
      },
      {
        kind: "rename",
        name: "local binding",
        anchor: anchorIn("const isListMode = computed(", "isListMode"),
        newName: "isListModeActive",
        min: 12,
      },
      { kind: "semanticTokens", name: "document", min: 1 },
    ],
  },
  ...SCALE_FIXTURES,
];

/** The SFC as the session opens it: probes planted, the script one broken. */
export function plantProbes(spec: FixtureSpec, source: string): string {
  const insert = (text: string, anchor: string, line: string) => {
    const first = text.indexOf(anchor);
    if (first < 0 || text.indexOf(anchor, first + 1) >= 0) {
      throw new Error(
        `${spec.id}: probe anchor must occur exactly once: ${JSON.stringify(anchor)}`,
      );
    }
    const at = first + anchor.length;
    return `${text.slice(0, at)}${line}${text.slice(at)}`;
  };
  const scripted = insert(source, spec.scriptProbeAfter, `${SCRIPT_BROKEN}\n`);
  return insert(
    scripted,
    spec.templateProbeAfter,
    `${spec.templateIndent}<i :data-bench="${TEMPLATE_CLEAN}"></i>\n`,
  );
}

interface RegistryEntry {
  id: string;
  repository: string;
  revision: string;
  typecheckPerformance?: { baseline?: { prepare?: string[] } };
}

function run(command: string, args: string[], cwd: string) {
  // Pinned projects declare engines for their own CI; editing needs no runtime match.
  const env = { ...process.env, npm_config_engine_strict: "false" };
  const result = spawnSync(command, args, { cwd, env, stdio: "inherit", timeout: 1_800_000 });
  if (result.status !== 0)
    throw new Error(`${command} ${args.join(" ")} failed in ${cwd} (${result.status})`);
}

/**
 * The fixture's project root, prepared once (`.lspbench-ready` makes reruns
 * free), and the edited SFC as its source has it: never the disk, which a
 * killed run may have left planted.
 */
export function prepareFixture(
  repoRoot: string,
  spec: FixtureSpec,
  fixturesDir: string,
): { root: string; original: string } {
  const root = resolve(fixturesDir, spec.id);
  if (spec.source.kind === "generated") {
    const files = prepareGenerated(repoRoot, root, spec.source.files);
    const original = files.get(spec.file);
    if (original == null) throw new Error(`${spec.id} generates no ${spec.file}`);
    return { root, original };
  }
  prepareRegistry(repoRoot, root, spec.id, spec.source.prepare);
  const show = spawnSync("git", ["show", `HEAD:${spec.file}`], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (show.status !== 0) throw new Error(`${spec.id}: git show ${spec.file}: ${show.stderr}`);
  return { root, original: show.stdout };
}

/** `name`'s pin in the workspace catalog `catalog` of pnpm-workspace.yaml. */
export function catalogVersion(workspaceYaml: string, catalog: string, name: string): string {
  const lines = workspaceYaml.split("\n");
  const start = lines.indexOf(`  ${catalog}:`);
  if (start < 0) throw new Error(`pnpm-workspace.yaml has no catalog ${catalog}`);
  for (const line of lines.slice(start + 1)) {
    if (line.trim() && !line.startsWith("    ")) break;
    const match = /^ {4}"?([^":]+)"?: "([^"]+)"$/.exec(line);
    if (match?.[1] === name) return match[2];
  }
  throw new Error(`catalog ${catalog} pins no ${name}`);
}

/** Order-independent digest of a generated project. */
export function projectDigest(files: Map<string, string>): string {
  const hash = createHash("sha256");
  for (const path of [...files.keys()].sort()) hash.update(`${path}\0${files.get(path)}\0`);
  return hash.digest("hex");
}

/** Write the generated files and install them with npm, Vue pinned by the workspace catalog. */
function prepareGenerated(
  repoRoot: string,
  root: string,
  generate: (vue: string) => Map<string, string>,
): Map<string, string> {
  const workspaceYaml = readFileSync(join(repoRoot, "pnpm-workspace.yaml"), "utf8");
  const files = generate(catalogVersion(workspaceYaml, "vue-production", "vue"));
  const digest = projectDigest(files);
  const stamp = join(root, ".lspbench-ready");
  if (existsSync(stamp) && readFileSync(stamp, "utf8").trim() === digest) return files;
  rmSync(root, { recursive: true, force: true });
  const made = new Set<string>();
  for (const [path, text] of files) {
    const target = join(root, path);
    if (!made.has(dirname(target))) mkdirSync(dirname(target), { recursive: true });
    made.add(dirname(target));
    writeFileSync(target, text);
  }
  run("npm", ["install", "--no-audit", "--no-fund", "--ignore-scripts", "--prefer-offline"], root);
  writeFileSync(stamp, `${digest}\n`);
  return files;
}

/**
 * Clone (shallow, pinned), install with the project's own pinned package
 * manager and run its preparation once.
 */
function prepareRegistry(
  repoRoot: string,
  root: string,
  id: string,
  prepareCommand: string[] | undefined,
): void {
  const registry = JSON.parse(
    readFileSync(join(repoRoot, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
  ) as { projects: RegistryEntry[] };
  const entry = registry.projects.find((project) => project.id === id);
  if (!entry) throw new Error(`${id} is not in the fixture registry`);
  const stamp = join(root, ".lspbench-ready");
  if (existsSync(stamp) && readFileSync(stamp, "utf8").trim() === entry.revision) return;
  if (!existsSync(root) || !readdirSync(root).some((name) => name !== ".git")) {
    mkdirSync(root, { recursive: true });
    run("git", ["init", "-q"], root);
    run("git", ["remote", "add", "origin", entry.repository], root);
    run("git", ["fetch", "-q", "--depth", "1", "origin", entry.revision], root);
    run("git", ["checkout", "-q", "FETCH_HEAD"], root);
  }
  const manifest = JSON.parse(readFileSync(join(root, "package.json"), "utf8")) as {
    packageManager?: string;
  };
  const pinned = /^(pnpm|yarn|npm)@([^+]+)/.exec(manifest.packageManager ?? "");
  if (!pinned || pinned[1] !== "pnpm") {
    throw new Error(`${id}: expected a pinned pnpm packageManager, got ${manifest.packageManager}`);
  }
  const pnpm = ["--yes", `pnpm@${pinned[2]}`];
  run(
    "npx",
    [...pnpm, "install", "--frozen-lockfile", "--ignore-scripts", "--prefer-offline"],
    root,
  );
  const prepare = prepareCommand ?? entry.typecheckPerformance?.baseline?.prepare?.slice(1);
  if (prepare) run("npx", [...pnpm, ...prepare], root);
  writeFileSync(stamp, `${entry.revision}\n`);
}

/** The deepest directory holding every one of `paths`. */
export function commonDirectory(paths: string[]): string {
  const parts = paths.map((path) => path.split(sep));
  let length = 0;
  while (length < parts[0].length && parts.every((other) => other[length] === parts[0][length]))
    length += 1;
  return parts[0].slice(0, length).join(sep);
}

/**
 * Declare the Vize content mapper in each of the fixture's tsconfigs and
 * install the mapper package once where all of them resolve it, for one
 * content-mapper session only: other editors must see the project exactly as
 * authored. Returns the restore step.
 */
export function installContentMapper(
  workspace: string,
  spec: FixtureSpec,
  mapperModules: string,
): () => void {
  const entry = '"contentMappers": [{ "package": "vize", "extensions": [".vue"] }],';
  const originals = new Map<string, string>();
  for (const relative of spec.tsconfigs) {
    const original = readFileSync(join(workspace, relative), "utf8");
    if (original.includes('"contentMappers"'))
      throw new Error(`${relative} already declares contentMappers`);
    if (original.indexOf("{") < 0) throw new Error(`${relative} is not a JSON object`);
    originals.set(join(workspace, relative), original);
  }
  for (const [tsconfig, original] of originals) {
    const open = original.indexOf("{") + 1;
    writeFileSync(tsconfig, `${original.slice(0, open)}\n  ${entry}${original.slice(open)}`);
  }
  const modules = join(commonDirectory([...originals.keys()].map(dirname)), "node_modules");
  mkdirSync(modules, { recursive: true });
  const added: string[] = [];
  for (const name of readdirSync(mapperModules)) {
    if (name.startsWith(".")) continue;
    // Scoped directories merge entry by entry; existing project packages win.
    const entries = name.startsWith("@")
      ? readdirSync(join(mapperModules, name)).map((inner) => join(name, inner))
      : [name];
    for (const entryName of entries) {
      const target = join(modules, entryName);
      if (existsSync(target)) continue;
      mkdirSync(dirname(target), { recursive: true });
      cpSync(join(mapperModules, entryName), target, { recursive: true, dereference: true });
      added.push(target);
    }
  }
  return () => {
    for (const target of added) rmSync(target, { recursive: true, force: true });
    for (const [tsconfig, original] of originals) writeFileSync(tsconfig, original);
  };
}
