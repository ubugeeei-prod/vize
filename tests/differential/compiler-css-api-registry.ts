export const cssMapProducerPaths = [
  "crates/vize_atelier_sfc/src/css.rs",
  "crates/vize_atelier_sfc/src/css/parser.rs",
  "crates/vize_atelier_sfc/src/css/parser/engine_boundary.rs",
  "crates/vize_atelier_sfc/src/css/parser/source_map.rs",
  "vendor/lightningcss/src/printer.rs",
] as const;

export const cssBindingProducerPaths = [
  ...cssMapProducerPaths.slice(0, 4),
  "crates/vize_atelier_sfc/src/css/parser/source_map/positions.rs",
  "crates/vize_atelier_sfc/src/css/parser/source_map/tests.rs",
  "crates/vize_croquis/src/sfc.rs",
  "crates/vize_croquis/src/sfc/css_transform.rs",
] as const;

type Spec = {
  id: string;
  root: string;
  adapter: string;
  source: string;
  options: Record<string, string | boolean>;
  filename: string;
  code: string;
  vars: string[];
  mappings: string;
  beforeRevision: string;
  fixRevision: string;
  paths: readonly string[];
  missingBefore: readonly number[];
  sourceBefore: boolean;
  witness: { runner: string; requestCount: number };
};

const specs: readonly Spec[] = [
  {
    id: "compiler/css/source-map-10693",
    root: "css-source-map-10693",
    adapter: "compile-css-original-input-map-10693",
    source: "\r\n.雪😀 { color: red }\r\n.b{opacity:0.5}",
    options: { filename: "/fixture/组件 space'\"&😀.css", minify: true, sourceMap: true },
    filename: "/fixture/组件 space'\"&😀.css",
    code: ".雪😀{color:red}.b{opacity:.5}",
    vars: [],
    mappings: "AACA,eACA",
    beforeRevision: "b0033883518f364cfbd3dd0ab9464715e9f51758",
    fixRevision: "510b0bb1033a22265e3408693a4aa00b78351cef",
    paths: cssMapProducerPaths,
    missingBefore: [3],
    sourceBefore: false,
    witness: { runner: "tests/native/css-compile-source-maps.mjs", requestCount: 84 },
  },
  {
    id: "compiler/css/v-bind-source-map-10712",
    root: "v-bind-source-map-10712",
    adapter: "compile-css-binding-map-10712",
    source: ".a{color:v-bind(theme)}.b{color:blue}",
    options: { scopeId: "data-v-123abc", minify: false, sourceMap: true },
    filename: "style.css",
    code: ".a {\n  color: var(--123abc-theme);\n}\n\n.b {\n  color: #00f;\n}\n",
    vars: ["theme"],
    mappings: "AAAA;;;;AAAuB",
    beforeRevision: "3a3c8460f966fddbd8e1be4c083d460b24202f90",
    fixRevision: "91c25c1b4a2ea842705e4db321b6e92d2b70ba42",
    paths: cssBindingProducerPaths,
    missingBefore: [4, 5],
    sourceBefore: true,
    witness: { runner: "tests/native/css-v-bind-source-maps.mjs", requestCount: 40 },
  },
];

export function compilerCssSpec(id: unknown): Spec | undefined {
  return specs.find((spec) => spec.id === id);
}

export function compilerCssApiSources(ids: readonly string[]) {
  return new Set(ids.flatMap((id) => compilerCssSpec(id)?.paths ?? []));
}

export function knownCompilerCssSource(path: string) {
  return specs.some((spec) => spec.paths.includes(path));
}
