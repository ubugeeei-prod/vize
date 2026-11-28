import { readFileSync } from "node:fs";
import { isVizeGeneratedVueModuleId, normalizeVizeGeneratedVueModuleId } from "./utils.ts";

/**
 * Nuxt 4.7 checks templates against the app (page routes, server components)
 * in a Vue compiler node transform: NUXT_B4025, NUXT_B3014 and NUXT_B3015.
 * Vize's compiler runs no node transforms, so the native side reads each SFC
 * the way that transform reads Vue's AST and returns what it would find. The
 * findings go back through Nuxt's own transform, as a template holding just
 * those constructs: the route match, the server component lookup, the layer
 * filter, the message, the dev overlay and the retiring of fixed warnings all
 * stay Nuxt's.
 */
export interface NuxtTemplateFinding {
  /** `link`, `event`, `prop` or `island`. */
  kind: string;
  tag: string;
  line: number;
  column: number;
  offset: number;
  attribute?: string | null;
  to?: string | null;
  directive?: string | null;
  prop?: string | null;
  value?: string | null;
  islandName?: string | null;
  islandProp?: string | null;
}

type NodeTransform = (node: unknown, context: unknown) => unknown;
type FindTemplateFindings = (source: string, serverTags: string[]) => NuxtTemplateFinding[];
type NuxtComponent = { mode?: string; pascalName?: string; kebabName?: string };

interface TemplateDiagnosticsHost {
  hook(name: string, callback: (...args: unknown[]) => unknown): void;
  apps?: { default?: { components?: NuxtComponent[] } };
  options: {
    experimental?: { templateDiagnostics?: boolean };
    vite?: { vue?: { template?: { compilerOptions?: { nodeTransforms?: unknown[] } } } };
  };
}

const ALWAYS_CHECKED_TAGS = ["NuxtLink", "nuxt-link", "NuxtIsland", "nuxt-island"];
const SFC_MODULE_ID = /\.vue(?:\.tsx?)?(?:\?|$)/;
const DESCRIBED_NODE: Record<string, string> = {
  "a function": "ArrowFunctionExpression",
  "a class": "ClassExpression",
  "a class instance": "NewExpression",
};

/**
 * Nuxt registers its template diagnostics transform in a `build:before` hook
 * added after every module's setup. A hook added in `modules:done` runs right
 * after it and finds it last among Nuxt's own `nodeTransform`s.
 */
export function captureNuxtTemplateDiagnostics(
  options: TemplateDiagnosticsHost["options"],
): NodeTransform | undefined {
  if (!options.experimental?.templateDiagnostics) return undefined;
  const transforms = options.vite?.vue?.template?.compilerOptions?.nodeTransforms ?? [];
  for (let index = transforms.length - 1; index >= 0; index--) {
    const transform = transforms[index];
    if (typeof transform === "function" && transform.name === "nodeTransform") {
      return transform as NodeTransform;
    }
  }
  return undefined;
}

/** Nuxt's `resolveServerComponentTags`: server-only components, every way a template names them. */
export function resolveServerComponentTags(components: readonly NuxtComponent[]): string[] {
  const tags = new Set<string>();
  for (const component of components) {
    if (
      component.mode !== "server" ||
      components.some(
        (other) => other.mode !== "server" && other.pascalName === component.pascalName,
      )
    ) {
      continue;
    }
    tags.add(`${component.pascalName}`);
    tags.add(`${component.kebabName}`);
    tags.add(`Lazy${component.pascalName}`);
    tags.add(`lazy-${component.kebabName}`);
  }
  return [...tags];
}

function location(finding: NuxtTemplateFinding) {
  const start = { line: finding.line, column: finding.column, offset: finding.offset };
  return { start, end: start, source: "" };
}

function staticArg(content: string) {
  return { type: 4, content, isStatic: true };
}

function describedValue(value: string | null | undefined) {
  return { type: DESCRIBED_NODE[value ?? ""] ?? "Unknown" };
}

/** The props of an element for which Nuxt's transform raises exactly `finding`. */
function findingProps(finding: NuxtTemplateFinding): unknown[] {
  const loc = location(finding);
  switch (finding.kind) {
    case "link":
      return [
        { type: 6, name: finding.attribute, value: { type: 2, content: finding.to, loc }, loc },
      ];
    case "event":
      return [{ type: 7, name: "model", rawName: finding.directive, modifiers: [], loc }];
    case "prop":
      return [
        {
          type: 7,
          name: "bind",
          rawName: `:${finding.prop}`,
          arg: staticArg(finding.prop ?? ""),
          exp: { type: 4, content: "", ast: describedValue(finding.value), loc },
          modifiers: [],
          loc,
        },
      ];
    case "island": {
      const property = {
        type: "ObjectProperty",
        computed: false,
        key: { type: "Identifier", name: finding.prop },
        value: describedValue(finding.value),
      };
      const name =
        finding.islandName == null
          ? []
          : [{ type: 6, name: "name", value: { type: 2, content: finding.islandName, loc }, loc }];
      return [
        ...name,
        {
          type: 7,
          name: "bind",
          rawName: `:${finding.islandProp}`,
          arg: staticArg(finding.islandProp ?? ""),
          exp: { type: 4, content: "", ast: { type: "ObjectExpression", properties: [property] } },
          modifiers: [],
          loc,
        },
      ];
    }
    default:
      return [];
  }
}

/** A compiler-core root with one component element per finding, in document order. */
export function toNuxtTemplateRoot(source: string, findings: readonly NuxtTemplateFinding[]) {
  return {
    type: 0,
    source,
    children: findings.map((finding) => ({
      type: 1,
      tag: finding.tag,
      tagType: 1,
      props: findingProps(finding),
      children: [],
      loc: location(finding),
    })),
  };
}

/** Run a node transform on a root as Vue's `transform()` does: enter, then its exit callbacks. */
export function runNodeTransform(transform: NodeTransform, root: unknown, context: unknown): void {
  const exit = transform(root, context);
  for (const callback of Array.isArray(exit) ? exit : [exit]) {
    if (typeof callback === "function") callback();
  }
}

function mentionsCheckedTag(source: string, serverTags: readonly string[]): boolean {
  return (
    ALWAYS_CHECKED_TAGS.some((tag) => source.includes(tag)) ||
    serverTags.some((tag) => source.includes(tag))
  );
}

interface TemplateDiagnosticsBridgeOptions {
  getTransform: () => NodeTransform | undefined;
  getServerTags: () => string[];
  loadFinder: () => Promise<FindTemplateFindings>;
  readSource?: (file: string) => string;
}

/** Report Nuxt's template diagnostics for each SFC Vize compiles, per environment. */
export function createNuxtTemplateDiagnosticsBridge(options: TemplateDiagnosticsBridgeOptions) {
  const readSource = options.readSource ?? ((file: string) => readFileSync(file, "utf8"));
  let finder: Promise<FindTemplateFindings> | undefined;
  return {
    name: "vizejs:nuxt-template-diagnostics",
    enforce: "post" as const,
    transform: {
      filter: { id: SFC_MODULE_ID },
      async handler(_code: string, id: string, transformOptions?: { ssr?: boolean }) {
        const transform = options.getTransform();
        if (!transform || !isVizeGeneratedVueModuleId(id)) return;
        const file = normalizeVizeGeneratedVueModuleId(id).split("?")[0]!;
        if (file.includes("/node_modules/")) return;
        let source: string;
        try {
          source = readSource(file);
        } catch {
          return;
        }
        const serverTags = options.getServerTags();
        // A template that names no checked tag has nothing to report, but
        // Nuxt still hears of the compile so a fixed warning is retired.
        const findings = mentionsCheckedTag(source, serverTags)
          ? (await (finder ||= options.loadFinder()))(source, serverTags)
          : [];
        const ssr = transformOptions?.ssr === true;
        try {
          runNodeTransform(transform, toNuxtTemplateRoot(source, findings), {
            filename: file,
            ssr,
            inSSR: ssr,
            bindingMetadata: {},
          });
        } catch {
          // A Nuxt release whose transform reads more than these fields stays silent.
        }
      },
    },
  };
}

/** Wire Nuxt's template diagnostics to the Vite plugins of a Vize-compiled app. */
export function setupNuxtTemplateDiagnostics(
  nuxt: TemplateDiagnosticsHost,
  plugins: unknown[],
): void {
  if (!nuxt.options.experimental?.templateDiagnostics) return;
  let transform: NodeTransform | undefined;
  nuxt.hook("modules:done", () => {
    nuxt.hook("build:before", () => {
      transform = captureNuxtTemplateDiagnostics(nuxt.options);
    });
  });
  let components: readonly NuxtComponent[] | undefined;
  let serverTags: string[] = [];
  plugins.push(
    createNuxtTemplateDiagnosticsBridge({
      getTransform: () => transform,
      getServerTags() {
        const current = nuxt.apps?.default?.components;
        if (current !== components) {
          components = current;
          serverTags = current ? resolveServerComponentTags(current) : [];
        }
        return serverTags;
      },
      async loadFinder() {
        const native = (await import("@vizejs/native")) as unknown as {
          nuxtTemplateDiagnostics?: FindTemplateFindings;
        };
        return native.nuxtTemplateDiagnostics ?? (() => []);
      },
    }),
  );
}
