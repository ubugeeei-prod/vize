const sfc = (script, template = "<p>Content</p>", setup = true) =>
  `<script${setup ? " setup" : ""} lang="ts">\n${script}\n</script>\n<template>${template}</template>`;
export function project(bad, good, root = "App.vue") {
  return {
    shared: {
      "main.ts": `import { createApp } from "vue";\nimport Root from "./${root}";\ncreateApp(Root).mount("#app");`,
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
    },
    bad,
    good: { ...bad, ...good },
  };
}
const child = (bad, good, template = "<Child />") =>
  project(
    {
      "App.vue": sfc('import Child from "./Child.vue";', template),
      "Child.vue": bad,
    },
    { "Child.vue": good },
  );
const props = "const props = defineProps<{ title: string }>();";
const emits = "const emit = defineEmits<{ save: [] }>();";
export const extra = {
  "browser-api-ssr": project(
    { "App.vue": sfc("const width = window.innerWidth;") },
    {
      "App.vue": sfc(
        'import { onMounted, ref } from "vue";\nconst width = ref(0);\nonMounted(() => { width.value = window.innerWidth; });',
      ),
    },
  ),
  "async-no-suspense": project(
    {
      "App.vue": sfc('import Child from "./Child.vue";', "<Child />"),
      "Child.vue": sfc('const greeting = await Promise.resolve("Hello");', "<p>{{ greeting }}</p>"),
    },
    {
      "App.vue": sfc(
        'import Child from "./Child.vue";',
        "<Suspense><Child /><template #fallback><p>Loading</p></template></Suspense>",
      ),
    },
  ),
  "uncaught-error": child(
    sfc('throw new Error("Request failed");'),
    sfc('throw new Error("Request failed");'),
  ),
  "provide-inject-type": child(
    sfc('import { inject } from "vue";\nconst title = inject<number>("title");'),
    sfc('import { inject } from "vue";\nconst title = inject<string>("title");'),
  ),
  "missing-required-prop": project(
    {
      "App.vue": sfc('import Child from "./Child.vue";', "<Child />"),
      "Child.vue": sfc(props, "<p>{{ props.title }}</p>"),
    },
    { "App.vue": sfc('import Child from "./Child.vue";', '<Child title="Hello" />') },
  ),
  "prop-type-mismatch": project(
    {
      "App.vue": sfc('import Child from "./Child.vue";', '<Child :title="42" />'),
      "Child.vue": sfc(props, "<p>{{ props.title }}</p>"),
    },
    { "App.vue": sfc('import Child from "./Child.vue";', '<Child title="Hello" />') },
  ),
  "undeclared-prop": project(
    {
      "App.vue": sfc('import Child from "./Child.vue";', '<Child title="Hello" :typo="true" />'),
      "Child.vue": sfc(props, "<p>{{ props.title }}</p>"),
    },
    { "App.vue": sfc('import Child from "./Child.vue";', '<Child title="Hello" />') },
  ),
  "undeclared-emit": child(
    sfc('const emit = defineEmits<{ cancel: [] }>();\nemit("save");'),
    sfc(`${emits}\nemit("save");`),
  ),
  "unused-emit": child(sfc(emits), sfc(`${emits}\nemit("save");`)),
  "unmatched-listener": project(
    {
      "App.vue": sfc('import Child from "./Child.vue";', '<Child @save="() => {}" />'),
      "Child.vue": sfc("const emit = defineEmits<{ cancel: [] }>();"),
    },
    { "Child.vue": sfc(`${emits}\nemit("save");`) },
  ),
  "event-modifier": project(
    {
      "App.vue": sfc('import Child from "./Child.vue";', '<Child @save.stop="() => {}" />'),
      "Child.vue": sfc(`${emits}\nemit("save");`),
    },
    { "App.vue": sfc('import Child from "./Child.vue";', '<Child @save="() => {}" />') },
  ),
  "unhandled-event": project(
    {
      "App.vue": sfc('import Wrapper from "./Wrapper.vue";', "<Wrapper />"),
      "Wrapper.vue": sfc('import Child from "./Child.vue";', "<Child />"),
      "Child.vue": sfc(`${emits}\nemit("save");`),
    },
    { "Wrapper.vue": sfc('import Child from "./Child.vue";', '<Child @save="() => {}" />') },
  ),
  "unregistered-component": project(
    {
      "App.vue": "<template><Child /></template>",
      "Child.vue": "<template><p>Child</p></template>",
    },
    { "App.vue": sfc('import Child from "./Child.vue";', "<Child />") },
  ),
  "unresolved-import": project(
    {
      "App.vue": sfc('import Child from "./Missing.vue";', "<Child />"),
      "Child.vue": "<template><p>Child</p></template>",
    },
    { "App.vue": sfc('import Child from "./Child.vue";', "<Child />") },
  ),
  "lifecycle-without-cleanup": project(
    {
      "App.vue": sfc(
        'import { onMounted } from "vue";\nconst resize = () => {};\nonMounted(() => { window.addEventListener("resize", resize); });',
      ),
    },
    {
      "App.vue": sfc(
        'import { onMounted, onUnmounted } from "vue";\nconst resize = () => {};\nonMounted(() => { window.addEventListener("resize", resize); });\nonUnmounted(() => { window.removeEventListener("resize", resize); });',
      ),
    },
  ),
  "setup-context-violation": project(
    {
      "App.vue": sfc(
        'import { ref } from "vue";\nconst count = ref(0);\nexport default {};',
        "<p>Count</p>",
        false,
      ),
    },
    { "App.vue": sfc('import { ref } from "vue";\nconst count = ref(0);', "<p>{{ count }}</p>") },
  ),
  "inherit-attrs-unused": child(
    sfc("defineOptions({ inheritAttrs: false });", "<main>Content</main>"),
    sfc("defineOptions({ inheritAttrs: false });", '<main v-bind="$attrs">Content</main>'),
    '<Child class="notice" />',
  ),
  "multi-root-attrs": child(
    "<template><main>Content</main><aside>Help</aside></template>",
    '<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>',
    '<Child class="notice" />',
  ),
  "unused-attrs": child(
    "<template><main>Content</main><aside>Help</aside></template>",
    '<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>',
    '<Child tracking-code="notice" />',
  ),
};
extra["uncaught-error"].good["App.vue"] = sfc(
  'import { onErrorCaptured } from "vue";\nimport Child from "./Child.vue";\nonErrorCaptured(() => false);',
  "<Child />",
);
for (const side of ["bad", "good"])
  extra["provide-inject-type"][side]["App.vue"] = sfc(
    'import { provide } from "vue";\nimport Child from "./Child.vue";\nprovide("title", "Hello");',
    "<Child />",
  );
export const composed = {
  "html/cross-component-nesting": project(
    {
      "App.vue": sfc('import Child from "./Child.vue";', "<p><Child /></p>"),
      "Child.vue": "<template><div>Block content</div></template>",
    },
    { "App.vue": sfc('import Child from "./Child.vue";', "<section><Child /></section>") },
  ),
  "vue/cross-file-attrs-fallthrough": child(
    "<template><main>Content</main><aside>Help</aside></template>",
    '<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>',
    '<Child class="notice" />',
  ),
};
