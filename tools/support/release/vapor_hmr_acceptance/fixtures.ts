/** Authored RouterView/KeepAlive reproduction of naitokosuke's #9180; not the reporter's app. */
export const vueVersion = "3.6.0-rc.11";
export const sources: Record<string, string> = {
  "index.html": '<div id="app"></div><script type="module" src="/src/main.js"></script>',
  "src/main.js": `import { createApp, vaporInteropPlugin, version } from 'vue';
import { createRouter, createWebHistory } from 'vue-router';
import App from './App.vue';
import Room from './Room.vue';
import Canvas from './Canvas.vue';
import Away from './Away.vue';
window.__acceptance = { vueVersion: version, events: [], errors: [], warnings: [], hmr: [] };
window.addEventListener('error', event => window.__acceptance.errors.push(String(event.error || event.message)));
window.addEventListener('unhandledrejection', event => window.__acceptance.errors.push(String(event.reason)));
for (const phase of ['beforeUpdate', 'afterUpdate', 'beforeFullReload']) {
  import.meta.hot.on('vite:' + phase, data => window.__acceptance.hmr.push({ phase, data }));
}
const router = createRouter({ history: createWebHistory(), routes: [
  { path: '/room', component: Room, children: [
    { path: 'canvas', component: Canvas }, { path: 'away', component: Away }
  ] }
] });
const app = createApp(App).use(vaporInteropPlugin).use(router);
app.config.errorHandler = error => window.__acceptance.errors.push(String(error));
app.config.warnHandler = warning => window.__acceptance.warnings.push(warning);
await router.isReady();
app.mount('#app');
window.__acceptance.ready = true;`,
  "src/App.vue": `<script setup>import { RouterView } from 'vue-router';</script>
<template><main><RouterView /></main></template>`,
  "src/Room.vue": `<script setup vapor>
import { computed, onUnmounted, ref } from 'vue';
import { RouterLink, RouterView } from 'vue-router';
const count = ref(0);
const inRoom = computed(() => count.value >= 0);
const label = computed(() => 'room-' + count.value);
window.__acceptance.events.push('parent-setup');
onUnmounted(() => window.__acceptance.events.push('parent-unmounted'));
</script>
<template><section v-if="inRoom" id="room">
<button id="increment" @click="count++">{{ count }}</button>
<RouterLink id="to-canvas" to="/room/canvas">Canvas</RouterLink>
<RouterLink id="to-away" to="/room/away">Away</RouterLink>
<RouterView v-slot="{ Component }"><KeepAlive><component :is="Component" :label="label" /></KeepAlive></RouterView>
</section></template>`,
  "src/Canvas.vue": `<script setup vapor>
defineProps({ label: String });
const revision = 'first';
</script><template><p id="canvas">{{ label }}:{{ revision }}</p></template>`,
  "src/Away.vue": `<script setup vapor>defineProps({ label: String });</script>
<template><p id="away">{{ label }}</p></template>`,
};
