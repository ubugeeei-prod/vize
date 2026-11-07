export interface RuntimeMetrics {
  setup: { App: number; Counter: number };
  childEvents: number;
  parentEvents: number;
  events: { owner: "App" | "Counter"; type: string; trusted: boolean }[];
}

// One authored TSX program is compiled independently for every host/format/target.
export const source = `import { ref } from "vue";
export const controls: { update?: () => void; restore?: () => void; resetCounter?: () => void } = {};
const metrics = globalThis.__vizeBabelRuntime;
export const Counter = (props: { label: string }) => {
  metrics.setup.Counter++;
  const count = ref(0);
  controls.resetCounter = () => { count.value = 0; };
  const increment = (event: MouseEvent) => {
    metrics.childEvents++;
    metrics.events.push({ owner: "Counter", type: event.type, trusted: event.isTrusted });
    count.value++;
  };
  return <button type="button" class="counter-own" style={{ fontWeight: "700" }} onClick={increment}>{props.label + ":" + count.value}</button>;
};
export const App = () => {
  metrics.setup.App++;
  const label = ref("雪 🧭");
  const tone = ref("parent-cold");
  const color = ref("blue");
  const buttonId = ref("counter-initial");
  const parentPress = (event: MouseEvent) => {
    metrics.parentEvents++;
    metrics.events.push({ owner: "App", type: event.type, trusted: event.isTrusted });
  };
  controls.update = () => { label.value = "更新 café"; tone.value = "parent-warm"; color.value = "red"; buttonId.value = "counter-updated"; };
  controls.restore = () => { label.value = "雪 🧭"; tone.value = "parent-cold"; color.value = "blue"; buttonId.value = "counter-initial"; controls.resetCounter?.(); };
  return <section id="fixture" data-state={label.value}><Counter label={label.value} id={buttonId.value} class={tone.value} style={{ color: color.value }} data-probe={label.value} onClick={parentPress}/></section>;
};
`.replaceAll("\n", "\r\n");
export const counterTemplate =
  '<button type="button" class="counter-own" style="font-weight:700" @click="increment">{{ props.label + ":" + count }}</button>';
export const appTemplate =
  '<section id="fixture" :data-state="label"><Counter :label="label" :id="buttonId" :class="tone" :style="{ color }" :data-probe="label" @click="parentPress"/></section>';
export const freshMetrics = (): RuntimeMetrics => ({
  setup: { App: 0, Counter: 0 },
  childEvents: 0,
  parentEvents: 0,
  events: [],
});

const setupSource = `
export const controls = {};
const metrics = globalThis.__vizeBabelRuntime;
function counterSetup(props) {
  metrics.setup.Counter++;
  const count = Vue.ref(0);
  controls.resetCounter = () => { count.value = 0; };
  const increment = event => {
    metrics.childEvents++;
    metrics.events.push({ owner: "Counter", type: event.type, trusted: event.isTrusted });
    count.value++;
  };
  return { props, count, increment };
}
function appSetup() {
  metrics.setup.App++;
  const label = Vue.ref("雪 🧭"), tone = Vue.ref("parent-cold"), color = Vue.ref("blue"), buttonId = Vue.ref("counter-initial");
  const parentPress = event => {
    metrics.parentEvents++;
    metrics.events.push({ owner: "App", type: event.type, trusted: event.isTrusted });
  };
  controls.update = () => { label.value = "更新 café"; tone.value = "parent-warm"; color.value = "red"; buttonId.value = "counter-updated"; };
  controls.restore = () => { label.value = "雪 🧭"; tone.value = "parent-cold"; color.value = "blue"; buttonId.value = "counter-initial"; controls.resetCounter?.(); };
  return { label, tone, color, buttonId, parentPress };
}
`;
export function referenceModule(target: "vdom" | "vapor", phase: "client" | "server") {
  if (target === "vdom")
    return `import * as Vue from "vue";\n${setupSource}
export const Counter = Vue.defineComponent({ props: ["label"], setup(props) {
  const state = counterSetup(props);
  return () => Vue.h("button", { type: "button", class: "counter-own", style: { fontWeight: "700" }, onClick: state.increment }, props.label + ":" + state.count.value);
} });
export const App = Vue.defineComponent({ setup() {
  const state = appSetup();
  return () => Vue.h("section", { id: "fixture", "data-state": state.label.value }, [Vue.h(Counter, { label: state.label.value, id: state.buttonId.value, class: state.tone.value, style: { color: state.color.value }, "data-probe": state.label.value, onClick: state.parentPress })]);
} });\n`;
  const server = phase === "server";
  return `import * as Vue from "vue";
import { ${server ? "ssrRender" : "render"} as counterRender } from "./reference-counter-${phase}.mjs";
import { ${server ? "ssrRender" : "render"} as appRender } from "./reference-app-${phase}.mjs";
${setupSource}
export const Counter = ${server ? "Vue.defineComponent" : "Vue.defineVaporComponent"}({ ${server ? "__vapor: true," : ""} props: ["label"], setup: counterSetup, ${server ? "ssrRender" : "render"}: counterRender });
export const App = ${server ? "Vue.defineComponent" : "Vue.defineVaporComponent"}({ ${server ? "__vapor: true," : ""} components: { Counter }, setup: appSetup, ${server ? "ssrRender" : "render"}: appRender });\n`;
}
