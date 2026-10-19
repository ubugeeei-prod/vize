// Mount a case's official and Vize output under one runtime and drive both
// through the same state changes and events; any difference in DOM, handler
// calls, warnings or errors makes the case differ.
import { baseState, childSources } from "./state.ts";
import { contextReads, firstLine, loadClient, upstreamClient, vue } from "./runtime.ts";

export interface Compiled {
  id: string;
  source: string;
  code: string;
  errors: string[];
}

const children: Record<string, unknown> = {};
for (const [name, source] of Object.entries(childSources)) {
  const module = await loadClient(upstreamClient(source).code);
  children[name] = vue.defineVaporComponent({ name, setup: () => ({}), render: module.render });
}

function describe(value: unknown): string {
  return value && typeof value === "object" && "type" in value
    ? `Event:${(value as Event).type}`
    : JSON.stringify(value);
}

const fixedKeys = ["e", "event", "tag", "is", "to", "dyn", "k", "key", "name", "n"];

async function run(code: string, log: string[]): Promise<string[]> {
  const snapshots: string[] = [];
  let module: Record<string, unknown>;
  try {
    module = await loadClient(code);
  } catch (error) {
    log.push(`LOAD-THROW ${firstLine(error)}`);
    return snapshots;
  }
  const base = baseState(
    (name) =>
      (...args) =>
        log.push(`${name}(${args.map(describe).join(",")})`),
  );
  for (const id of contextReads(code)) {
    if (!(id in base) && id !== "vaporTemplateRefSetter") base[id] = `<${id}>`;
  }
  const state = vue.reactive(base);
  const app = vue.createVaporApp(
    vue.defineVaporComponent({ setup: () => state, render: module.render }),
  );
  for (const [name, component] of Object.entries(children)) app.component(name, component);
  app.config.errorHandler = (error: unknown) => log.push(`ERR ${firstLine(error)}`);
  app.config.warnHandler = (message: string) => log.push(`WARN ${firstLine(message)}`);
  const host = document.body.appendChild(document.createElement("div"));
  const snap = async () => {
    await vue.nextTick();
    snapshots.push(host.innerHTML);
  };
  const flipBooleans = () => {
    for (const [key, value] of Object.entries(state)) {
      if (typeof value === "boolean") state[key] = !value;
    }
  };
  try {
    app.mount(host);
    await snap();
    flipBooleans();
    await snap();
    for (const value of Object.values(state)) if (Array.isArray(value)) value.reverse();
    state.list = [...(state.list as unknown[]), 4];
    await snap();
    for (const [key, value] of Object.entries(state)) {
      if (typeof value === "string" && !fixedKeys.includes(key)) state[key] = `${value}!`;
    }
    await snap();
    for (const type of ["click", "mouseenter", "input", "keyup", "submit"]) {
      for (const element of Array.from(host.querySelectorAll("*"))) {
        element.dispatchEvent(
          type === "keyup"
            ? new KeyboardEvent("keyup", { key: "Enter", bubbles: true })
            : new Event(type, { bubbles: true, cancelable: true }),
        );
      }
    }
    await snap();
    flipBooleans();
    await snap();
  } catch (error) {
    log.push(`THROW ${firstLine(error)}`);
  } finally {
    try {
      app.unmount();
    } catch (error) {
      log.push(`UNMOUNT-THROW ${firstLine(error)}`);
    }
    host.remove();
  }
  return snapshots;
}

/** Whether a case behaves differently; both refusing to compile is agreement. */
export async function mountDiffers(compiled: Compiled): Promise<boolean> {
  const upstream = upstreamClient(compiled.source);
  const upstreamFails = upstream.errors.length > 0 || !upstream.code;
  const vizeFails = compiled.errors.length > 0 || !compiled.code;
  if (upstreamFails || vizeFails) return upstreamFails !== vizeFails;
  const upstreamLog: string[] = [];
  const vizeLog: string[] = [];
  const upstreamDom = await run(upstream.code, upstreamLog);
  const vizeDom = await run(compiled.code, vizeLog);
  return (
    JSON.stringify(upstreamDom) !== JSON.stringify(vizeDom) ||
    JSON.stringify(upstreamLog) !== JSON.stringify(vizeLog)
  );
}
