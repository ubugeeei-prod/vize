/** Runs in Chromium, with a fresh component for each Rust-owned scenario. */
export async function mountEventFixture({ code, runtimeUrl, backend, fixture }) {
  const vue = await import(runtimeUrl);
  const body = code
    .replace(/from\s*["']vue["']/gu, `from "${runtimeUrl}"`)
    .replace(/\bexport\s+function\s+render\s*\(/u, "function render(");
  const compiled = `import * as Vue from "${runtimeUrl}";\n${body}\nexport { render };`;
  const moduleUrl = URL.createObjectURL(new Blob([compiled], { type: "text/javascript" }));
  const { render } = await import(moduleUrl);
  URL.revokeObjectURL(moduleUrl);
  const events = [];
  const diagnostics = [];
  let phase = "mount";
  let firstNode;
  const host = document.createElement("div");
  document.body.append(host);
  const state = vue.reactive({ count: 0 });

  function replace(generation) {
    const save = (...args) => {
      const values = args[0] instanceof Event ? [args[0].type, args[0].target.id] : args;
      events.push([generation, values]);
    };
    state.save = save;
    state.actions = { save };
    switch (fixture.setup) {
      case "function":
        state.$event = save;
        break;
      case "member":
        state.$event = { target: save };
        break;
      case "value":
        state.$event = { type: `setup-${generation}`, target: `binding-${generation}` };
        break;
      case "null":
        state.$event = null;
        break;
      case "number":
        state.$event = 42;
        break;
      case "missing":
        break;
      default:
        throw new Error(`unknown setup: ${fixture.setup}`);
    }
  }
  replace("a");
  const cache = [];
  const component =
    backend === "vapor"
      ? vue.defineVaporComponent({ setup: () => render(state) })
      : { setup: () => () => render(state, cache, {}, state, {}, {}) };
  const app = (backend === "vapor" ? vue.createVaporApp : vue.createApp)(component);
  app.config.warnHandler = (message) =>
    diagnostics.push([
      phase,
      message.startsWith("Wrong type passed as event handler") ? "invalid-handler" : message,
    ]);
  app.config.errorHandler = (error) => diagnostics.push([phase, error.name]);

  function snapshot() {
    const node = host.querySelector("button");
    return {
      phase,
      count: state.count,
      text: node?.textContent ?? null,
      sibling: host.querySelector("span")?.textContent ?? null,
      present: !!node,
      connected: !!firstNode?.isConnected,
      sameNode: !!node && node === firstNode,
      events: structuredClone(events),
      diagnostics: structuredClone(diagnostics),
    };
  }
  app.mount(host);
  await vue.nextTick();
  firstNode = host.querySelector("button");
  window.eventFixture = {
    async step(next) {
      phase = next;
      if (next === "replace") replace("b");
      if (next === "unmount") app.unmount();
      await vue.nextTick();
      const result = snapshot();
      if (next === "unmount") host.remove();
      return result;
    },
    snapshot,
  };
  return snapshot();
}
