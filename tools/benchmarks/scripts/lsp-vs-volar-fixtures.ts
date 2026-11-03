/**
 * Real projects for `lsp-vs-volar.ts` at the registry's pinned revisions, and
 * the authored spots the scripted session edits and queries in each.
 *
 * Every anchor is unique text in the pinned file; a missing or repeated anchor
 * throws instead of measuring a guessed position. Expectations are what a
 * correct answer must contain, so a fast empty or wrong answer counts as a
 * failure rather than a win.
 */

import { spawnSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";

import type { Anchor } from "./lsp-vs-volar-text.ts";

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
  /** The tsconfig that owns `file`; the content-mapper editor declares the mapper there. */
  tsconfig: string;
  /** Commands run once after install, before any session. */
  prepare?: string[];
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

const after = (text: string, at: string) => ({ find: text, at: text.indexOf(at) });

export const FIXTURES: FixtureSpec[] = [
  {
    id: "misskey",
    file: "packages/frontend/src/components/MkNote.vue",
    tsconfig: "packages/frontend/tsconfig.json",
    scriptProbeAfter: '<script lang="ts" setup>\n',
    templateProbeAfter:
      '\t<MkNoteSub v-if="appearNote.replyId && !renoteCollapsed" :note="appearNote?.reply ?? null" :class="$style.replyTo"/>\n',
    templateIndent: "\t",
    requests: [
      {
        kind: "hover",
        name: "script binding",
        anchor: after("const appearNote = getAppearNote(note) ?? note;", "appearNote"),
        expect: ["appearNote:"],
      },
      {
        kind: "hover",
        name: "template expression",
        anchor: after("{{ appearNote.channel.name }}", "appearNote"),
        expect: ["appearNote:"],
      },
      {
        kind: "hover",
        name: "component tag",
        anchor: after('<MkNoteHeader :note="appearNote" :mini="true"/>', "MkNoteHeader"),
        expect: ["MkNoteHeader"],
      },
      {
        kind: "hover",
        name: "component prop",
        anchor: after('<MkNoteHeader :note="appearNote" :mini="true"/>', "note="),
        expect: ["note"],
      },
      {
        kind: "completion",
        name: "script member",
        anchor: after("computed(() => prefer.s.showSoftWordMutedWord)", "showSoftWordMutedWord"),
        expect: ["showSoftWordMutedWord"],
      },
      {
        kind: "completion",
        name: "template member",
        anchor: after("{{ appearNote.channel.name }}", "channel"),
        expect: ["channel", "user"],
      },
      {
        kind: "definition",
        name: "component tag",
        anchor: after('<MkNoteHeader :note="appearNote" :mini="true"/>', "MkNoteHeader"),
        expectPath: "src/components/MkNoteHeader.vue",
      },
      {
        kind: "definition",
        name: "imported function",
        anchor: after("const appearNote = getAppearNote(note) ?? note;", "getAppearNote"),
        expectPath: "src/utility/get-appear-note.ts",
      },
      {
        kind: "references",
        name: "local binding",
        anchor: after("const renoteCollapsed = ref(", "renoteCollapsed"),
        min: 12,
      },
      {
        kind: "references",
        name: "imported function",
        anchor: after(
          "import { getAppearNote } from '@/utility/get-appear-note.js';",
          "getAppearNote",
        ),
        min: 3,
      },
      {
        kind: "rename",
        name: "local binding",
        anchor: after("const renoteCollapsed = ref(", "renoteCollapsed"),
        newName: "renoteFolded",
        min: 12,
      },
      { kind: "semanticTokens", name: "document", min: 1 },
    ],
  },
  {
    id: "n8n",
    file: "packages/frontend/editor-ui/src/features/ndv/parameters/components/ResourceLocator/ResourceLocator.vue",
    tsconfig: "packages/frontend/editor-ui/tsconfig.json",
    // n8n's own recipe for its editor's workspace dependencies without full type builds.
    prepare: ["exec", "turbo", "run", "build:unchecked", "--filter=n8n-editor-ui^..."],
    scriptProbeAfter: '<script setup lang="ts">\n',
    templateProbeAfter: '\t\t:data-test-id="`resource-locator-${parameter.name}`"\n\t>\n',
    templateIndent: "\t\t",
    requests: [
      {
        kind: "hover",
        name: "script binding",
        anchor: after("const currentQueryResults = computed<", "currentQueryResults"),
        expect: ["currentQueryResults:"],
      },
      {
        kind: "hover",
        name: "template expression",
        anchor: after(':resources="currentQueryResults"', "currentQueryResults"),
        expect: ["currentQueryResults:"],
      },
      {
        kind: "hover",
        name: "component tag",
        anchor: after("<ResourceLocatorDropdown\n", "ResourceLocatorDropdown"),
        expect: ["ResourceLocatorDropdown"],
      },
      {
        kind: "hover",
        name: "component prop",
        anchor: after(':resources="currentQueryResults"', "resources"),
        expect: ["resources"],
      },
      {
        kind: "completion",
        name: "script member",
        anchor: after("if (currentMode.value.placeholder) {", "placeholder"),
        expect: ["placeholder"],
      },
      {
        kind: "completion",
        name: "template member",
        anchor: after("{{ currentResponse.errorDetails.httpCode }}", "errorDetails"),
        expect: ["errorDetails"],
      },
      {
        kind: "definition",
        name: "component tag",
        anchor: after("<ResourceLocatorDropdown\n", "ResourceLocatorDropdown"),
        expectPath: "ResourceLocator/ResourceLocatorDropdown.vue",
      },
      {
        kind: "definition",
        name: "imported function",
        anchor: after("const i18n = useI18n();", "useI18n"),
        expectPath: "@n8n/i18n/src/index.ts",
      },
      {
        kind: "references",
        name: "local binding",
        anchor: after("const isListMode = computed(", "isListMode"),
        min: 12,
      },
      {
        kind: "references",
        name: "imported function",
        anchor: after(
          "import { ndvEventBus } from '@/features/ndv/shared/ndv.eventBus';",
          "ndvEventBus",
        ),
        min: 3,
      },
      {
        kind: "rename",
        name: "local binding",
        anchor: after("const isListMode = computed(", "isListMode"),
        newName: "isListModeActive",
        min: 12,
      },
      { kind: "semanticTokens", name: "document", min: 1 },
    ],
  },
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
 * Clone (shallow, pinned), install with the project's own pinned package
 * manager and run its preparation once; `.lspbench-ready` makes reruns free.
 */
export function prepareFixture(repoRoot: string, spec: FixtureSpec, fixturesDir: string): string {
  const registry = JSON.parse(
    readFileSync(join(repoRoot, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
  ) as { projects: RegistryEntry[] };
  const entry = registry.projects.find((project) => project.id === spec.id);
  if (!entry) throw new Error(`${spec.id} is not in the fixture registry`);
  const root = resolve(fixturesDir, spec.id);
  const stamp = join(root, ".lspbench-ready");
  if (existsSync(stamp) && readFileSync(stamp, "utf8").trim() === entry.revision) return root;
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
    throw new Error(
      `${spec.id}: expected a pinned pnpm packageManager, got ${manifest.packageManager}`,
    );
  }
  const pnpm = ["--yes", `pnpm@${pinned[2]}`];
  run(
    "npx",
    [...pnpm, "install", "--frozen-lockfile", "--ignore-scripts", "--prefer-offline"],
    root,
  );
  const prepare = spec.prepare ?? entry.typecheckPerformance?.baseline?.prepare?.slice(1);
  if (prepare) run("npx", [...pnpm, ...prepare], root);
  writeFileSync(stamp, `${entry.revision}\n`);
  return root;
}

/**
 * Declare the Vize content mapper in the fixture's tsconfig and install the
 * mapper package beside it, for one content-mapper session only: other editors
 * must see the project exactly as authored. Returns the restore step.
 */
export function installContentMapper(
  workspace: string,
  spec: FixtureSpec,
  mapperModules: string,
): () => void {
  const tsconfig = join(workspace, spec.tsconfig);
  const original = readFileSync(tsconfig, "utf8");
  if (original.includes('"contentMappers"'))
    throw new Error(`${spec.tsconfig} already declares contentMappers`);
  const open = original.indexOf("{");
  if (open < 0) throw new Error(`${spec.tsconfig} is not a JSON object`);
  const entry = '"contentMappers": [{ "package": "vize", "extensions": [".vue"] }],';
  writeFileSync(tsconfig, `${original.slice(0, open + 1)}\n  ${entry}${original.slice(open + 1)}`);
  const modules = join(dirname(tsconfig), "node_modules");
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
    writeFileSync(tsconfig, original);
  };
}
