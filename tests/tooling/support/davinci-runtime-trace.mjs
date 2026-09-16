import process from "node:process";
import { pathToFileURL } from "node:url";

const importPattern = /import\s*\{([^}]+)\}\s*from\s*["']vue["'];?/gu;

export async function evaluateCompiledRender(code, runtime) {
  const { body, bindings } = stripVueImports(code);
  return evaluateRender(body, runtime, bindings, {});
}

export async function traceCompiledBackend({ backend, code, context = {} }) {
  const trace = [];
  const renderContext = { $slots: {}, save: noop, ...context };
  const helpers = backend === "vdom" ? createVdomHelpers() : createVaporHelpers(trace);
  const { body, bindings } = stripVueImports(code);
  const render = await evaluateRender(body, helpers, bindings, renderContext);

  if (backend === "vdom") {
    const vnode = render(renderContext, [], {}, {}, {}, {});
    mountVdom(vnode, trace);
  } else {
    render(renderContext);
  }

  return trace;
}

function stripVueImports(code) {
  const bindings = [];
  const body = code.replace(importPattern, (_, imports) => {
    for (const specifier of imports.split(",")) {
      const trimmed = specifier.trim();
      if (!trimmed) continue;

      const [imported, local = imported] = trimmed.split(/\s+as\s+/u);
      const importedName = imported.trim();
      const localName = local.trim();
      if (!isIdentifierName(importedName) || !isIdentifierName(localName)) {
        throw new Error(`unsupported Vue import binding: ${trimmed}`);
      }
      bindings.push([localName, importedName]);
    }
    return "";
  });

  return { body, bindings };
}

async function evaluateRender(code, helpers, bindings, context) {
  const rewritten = code
    .replace(/\bexport\s+function\s+render\s*\(/u, "function render(")
    .replace(/\bexport\s+const\s+render\s*=/u, "const render =")
    .replace(/\bexport\s+\{[^}]+\};?/gu, "");
  const stateKey = `__davinciRuntimeTraceState_${process.pid}_${Date.now()}_${Math.random()
    .toString(36)
    .slice(2)}`;
  const lexicalContext = Object.entries(context).filter(([name]) => isIdentifierName(name));
  const helperDeclarations = bindings
    .map(([localName, importedName]) => {
      return `const ${localName} = __state.helperValue(${JSON.stringify(importedName)});`;
    })
    .join("\n");
  const contextDeclarations = lexicalContext
    .map(([name]) => `const ${name} = __state.context[${JSON.stringify(name)}];`)
    .join("\n");
  const moduleSource = [
    `const __state = globalThis[${JSON.stringify(stateKey)}];`,
    "const Vue = __state.helpers;",
    helperDeclarations,
    contextDeclarations,
    rewritten,
    "export { render };",
  ].join("\n");
  globalThis[stateKey] = { context, helperValue: (name) => helperValue(helpers, name), helpers };

  let module;
  try {
    const encoded = Buffer.from(moduleSource, "utf8").toString("base64");
    module = await import(`data:text/javascript;base64,${encoded}`);
  } finally {
    delete globalThis[stateKey];
  }
  const render = module.render;

  if (typeof render !== "function") {
    throw new Error("compiled module did not export render()");
  }
  return render;
}

function helperValue(helpers, name) {
  if (Object.prototype.hasOwnProperty.call(helpers, name)) {
    return helpers[name];
  }
  throw new Error(`unsupported vue runtime helper: ${name}`);
}

function isIdentifierName(name) {
  return /^[$A-Z_a-z][$\w]*$/u.test(name);
}

function createVdomHelpers() {
  const createElement = (
    tag,
    props = null,
    children = null,
    patchFlag = 0,
    dynamicProps = null,
  ) => ({
    kind: "element",
    tag,
    props,
    children,
    patchFlag,
    dynamicProps,
    branch: props && Object.prototype.hasOwnProperty.call(props, "key"),
  });

  return {
    Fragment: Symbol("Fragment"),
    Text: Symbol("Text"),
    Comment: Symbol("Comment"),
    createBlock: createElement,
    createCommentVNode: () => ({ kind: "comment" }),
    createElementBlock: createElement,
    createElementVNode: createElement,
    createTextVNode: (text = "") => ({ kind: "text", text }),
    createVNode: createElement,
    normalizeClass: (value) => value,
    normalizeStyle: (value) => value,
    openBlock: noop,
    renderList: (source, callback) => Array.from(source, callback),
    renderSlot: (_slots, _name, _props, fallback) => ({
      kind: "slot",
      children: typeof fallback === "function" ? fallback() : [],
    }),
    resolveComponent: (name) => name,
    toDisplayString: (value) => String(value ?? ""),
    withCtx: (fn) => fn,
    withDirectives: (vnode) => vnode,
  };
}

function mountVdom(node, trace) {
  if (node == null || typeof node === "boolean") return;
  if (Array.isArray(node)) {
    for (const child of node) mountVdom(child, trace);
    return;
  }
  if (typeof node === "string" || typeof node === "number") {
    trace.push("set-text");
    return;
  }
  if (node.kind === "comment") return;
  if (node.kind === "text") {
    trace.push("set-text");
    return;
  }
  if (node.kind === "slot") {
    trace.push("render-slot");
    mountVdom(node.children, trace);
    return;
  }
  if (node.kind !== "element") return;

  if (node.branch) trace.push("branch");
  trace.push("create-element");
  recordDynamicVdomProps(node, trace);
  if (node.props && Object.prototype.hasOwnProperty.call(node.props, "textContent")) {
    trace.push("set-text");
  } else {
    mountVdom(node.children, trace);
  }
}

function recordDynamicVdomProps(node, trace) {
  const dynamicProps = Array.isArray(node.dynamicProps) ? node.dynamicProps : [];
  for (const prop of dynamicProps) {
    if (prop === "textContent") continue;
    trace.push(/^on[A-Z]/u.test(prop) ? "patch-event" : "patch-prop");
  }
}

function createVaporHelpers(trace) {
  return {
    child: () => createHostNode(trace),
    createFor: (_source, renderItem) => {
      trace.push("list-effect");
      return typeof renderItem === "function" ? renderItem(undefined, 0) : createHostNode(trace);
    },
    createIf: (condition, factory) => {
      trace.push("conditional-effect");
      return condition() ? factory() : createHostNode(trace);
    },
    createInvoker: (handler) => handler,
    createSlot: (_name, _props, fallback) => {
      trace.push("slot-effect");
      return typeof fallback === "function" ? fallback() : createHostNode(trace);
    },
    delegateEvents: noop,
    insert: noop,
    next: () => createHostNode(trace),
    on: () => {
      trace.push("listen");
    },
    renderEffect: (effect) => effect(),
    setClass: () => {
      trace.push("assign-prop");
    },
    setDynamicProps: () => {
      trace.push("assign-dynamic-props");
    },
    setInsertionState: noop,
    setProp: () => {
      trace.push("assign-prop");
    },
    setText: () => {
      trace.push("text-effect");
    },
    setElementText: () => {
      trace.push("text-effect");
    },
    template: (html) => () => {
      for (const label of vaporTemplateLabels(html)) trace.push(label);
      return createHostNode(trace);
    },
    toDisplayString: (value) => String(value ?? ""),
    txt: () => createHostNode(trace),
  };
}

function createHostNode(trace) {
  return new Proxy(
    {},
    {
      set(target, property, value) {
        if (typeof property === "string" && property.startsWith("$evt")) {
          trace.push("listen");
        }
        target[property] = value;
        return true;
      },
    },
  );
}

function vaporTemplateLabels(template) {
  const labels = [];
  let offset = 0;

  while (true) {
    const tagStart = template.indexOf("<", offset);
    if (tagStart === -1) break;

    if (template.slice(offset, tagStart).trim()) {
      labels.push("text-effect");
    }
    const next = template.at(tagStart + 1);
    if (next !== "/" && next !== "!" && next !== "?") {
      labels.push("create-node");
    }

    const tagEnd = template.indexOf(">", tagStart);
    if (tagEnd === -1) return labels;
    offset = tagEnd + 1;
  }

  if (template.slice(offset).trim()) {
    labels.push("text-effect");
  }
  return labels;
}

function noop() {}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const chunks = [];
  for await (const chunk of process.stdin) {
    chunks.push(chunk);
  }
  const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  process.stdout.write(`${JSON.stringify(await traceCompiledBackend(input))}\n`);
}
