// Server-render a case with the official SSR compiler (`vapor: true`), then
// hydrate that markup with the official and the Vize client output and
// compare the reused nodes, the DOM after updates, and the logs.
import { baseState, childSources } from "./state.ts";
import type { Compiled } from "./mount.ts";
import {
  compilerSsr,
  contextReads,
  firstLine,
  loadClient,
  loadServer,
  serverRenderer,
  serverVue,
  upstreamClient,
  vue,
} from "./runtime.ts";

/** Sources whose server render depends on more than this harness provides. */
const unsupported =
  /\$attrs|\$slots|\$emit|Teleport|teleport|v-memo|: MouseEvent|: Event|as any|as string|foo!|<T,>|: any|: \{ a: string \}/;

const ssrOptions = {
  prefixIdentifiers: true,
  mode: "module" as const,
  vapor: true,
  comments: false,
  onWarn: () => {},
};

const serverChildren: Record<string, unknown> = {};
const clientChildren: Record<string, unknown> = {};
for (const [name, source] of Object.entries(childSources)) {
  const server = await loadServer(compilerSsr.compile(source, ssrOptions).code);
  serverChildren[name] = { name, __vapor: true, setup: () => ({}), ssrRender: server.ssrRender };
  const client = await loadClient(upstreamClient(source).code);
  clientChildren[name] = vue.defineVaporComponent({
    name,
    setup: () => ({}),
    render: client.render,
  });
}

async function serverHtml(source: string): Promise<string | undefined> {
  const errors: string[] = [];
  let code: string;
  try {
    code = compilerSsr.compile(source, {
      ...ssrOptions,
      onError: (e: Error) => errors.push(e.message),
    }).code;
  } catch {
    return undefined;
  }
  if (errors.length) return undefined;
  let module: Record<string, unknown>;
  try {
    module = await loadServer(code);
  } catch {
    return undefined;
  }
  const state = baseState(() => () => {});
  for (const id of contextReads(code)) if (!(id in state)) state[id] = `<${id}>`;
  const app = serverVue.createSSRApp({
    __vapor: true,
    setup: () => state,
    ssrRender: module.ssrRender,
  });
  for (const [name, component] of Object.entries(serverChildren)) app.component(name, component);
  app.config.warnHandler = () => {};
  try {
    return await serverRenderer.renderToString(app);
  } catch {
    return undefined;
  }
}

interface Outcome {
  log: string[];
  kept: string;
  snapshots: string[];
}

async function hydrate(code: string, html: string): Promise<Outcome> {
  const log: string[] = [];
  const snapshots: string[] = [];
  let module: Record<string, unknown>;
  try {
    module = await loadClient(code);
  } catch (error) {
    return { log: [`LOAD-THROW ${firstLine(error)}`], kept: "", snapshots };
  }
  const base = baseState(
    (name) =>
      (...args) =>
        log.push(`${name}(${args.length})`),
  );
  for (const id of contextReads(code)) {
    if (!(id in base) && id !== "vaporTemplateRefSetter") base[id] = `<${id}>`;
  }
  const state = vue.reactive(base);
  const host = document.body.appendChild(document.createElement("div"));
  host.innerHTML = html;
  const serverElements = [...host.querySelectorAll("*")];
  const app = vue.createVaporSSRApp(
    vue.defineVaporComponent({ setup: () => state, render: module.render }),
  );
  for (const [name, component] of Object.entries(clientChildren)) app.component(name, component);
  app.config.errorHandler = (error: unknown) => log.push(`ERR ${firstLine(error)}`);
  app.config.warnHandler = (message: string) => log.push(`WARN ${firstLine(message)}`);
  const { warn, error } = console;
  console.warn = (...args: unknown[]) => log.push(`CWARN ${firstLine(args[0])}`);
  console.error = (...args: unknown[]) => log.push(`CERR ${firstLine(args[0])}`);
  let kept = "";
  const snap = async () => {
    await vue.nextTick();
    snapshots.push(host.innerHTML);
  };
  try {
    app.mount(host);
    await snap();
    kept = `${serverElements.filter((element) => host.contains(element)).length}/${serverElements.length}`;
    for (const [key, value] of Object.entries(state)) {
      if (typeof value === "boolean") state[key] = !value;
    }
    await snap();
    for (const value of Object.values(state)) if (Array.isArray(value)) value.reverse();
    state.list = [...(state.list as unknown[]), 4];
    await snap();
    for (const element of Array.from(host.querySelectorAll("*"))) {
      element.dispatchEvent(new Event("click", { bubbles: true, cancelable: true }));
    }
    await snap();
  } catch (thrown) {
    log.push(`THROW ${firstLine(thrown)}`);
  } finally {
    console.warn = warn;
    console.error = error;
    try {
      app.unmount();
    } catch (thrown) {
      log.push(`UNMOUNT-THROW ${firstLine(thrown)}`);
    }
    host.remove();
  }
  return { log, kept, snapshots };
}

/**
 * Whether hydrating a case differs, or `undefined` when it does not apply:
 * the harness cannot server-render it or either compiler refuses it.
 */
export async function hydrateDiffers(compiled: Compiled): Promise<boolean | undefined> {
  if (unsupported.test(compiled.source)) return undefined;
  const html = await serverHtml(compiled.source);
  if (html === undefined) return undefined;
  const upstream = upstreamClient(compiled.source);
  if (upstream.errors.length || !upstream.code || compiled.errors.length || !compiled.code) {
    return undefined;
  }
  const expected = await hydrate(upstream.code, html);
  const actual = await hydrate(compiled.code, html);
  return JSON.stringify(expected) !== JSON.stringify(actual);
}
