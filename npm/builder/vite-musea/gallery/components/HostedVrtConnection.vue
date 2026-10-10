<script setup lang="ts">
import { ref, onMounted } from "vue";
import { joinBasePath } from "../staticApi";

defineProps<{ connected: boolean; error: string }>();
const emit = defineEmits<{ connect: [endpoint: string, token: string] }>();
const endpoint = ref("");
const token = ref("");
const command = ref("");
onMounted(() => {
  const url = new URL(joinBasePath("/"), window.location.href).href;
  command.value = `vp exec musea-vrt serve --gallery-url '${url.replaceAll("'", "'\"'\"'")}'`;
});
function connect() {
  emit("connect", endpoint.value.trim(), token.value.trim());
  token.value = "";
}
</script>

<template>
  <form class="hosted-vrt-connect" @submit.prevent="connect">
    <p v-if="connected">Connected to your local VRT session.</p>
    <template v-else>
      <p>Start a local VRT session, then connect to capture and review this gallery.</p>
      <code>{{ command }}</code>
      <label
        >VRT endpoint
        <input v-model="endpoint" type="url" placeholder="http://127.0.0.1:…" required
      /></label>
      <label
        >Session token <input v-model="token" type="password" autocomplete="off" required
      /></label>
      <button type="submit">Connect VRT</button>
    </template>
    <p v-if="error" role="alert">{{ error }}</p>
  </form>
</template>

<style scoped>
.hosted-vrt-connect {
  display: grid;
  gap: 0.65rem;
  margin-bottom: 1rem;
  font-size: 0.8125rem;
}
code {
  overflow-wrap: anywhere;
}
label {
  display: grid;
  gap: 0.25rem;
}
input {
  padding: 0.5rem;
  background: var(--musea-bg-secondary);
  color: var(--musea-text);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
}
button {
  justify-self: start;
  padding: 0.5rem 0.75rem;
  border: 0;
  border-radius: var(--musea-radius-sm);
  background: var(--musea-accent);
  color: var(--musea-accent-contrast);
  cursor: pointer;
}
[role="alert"] {
  color: var(--musea-error);
}
</style>
