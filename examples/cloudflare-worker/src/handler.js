const DEFAULT_SOURCE = `<template>
  <button @click="count++">Count: {{ count }}</button>
</template>

<script setup lang="ts">
import { ref } from "vue";

const count = ref<number>(0);
</script>`;

const MAX_SOURCE_BYTES = 1024 * 1024;

function errorResponse(error, status) {
  return Response.json({ ok: false, error }, { status });
}

async function inputFromRequest(request) {
  if (request.method === "GET") {
    return { source: DEFAULT_SOURCE, options: { filename: "Counter.vue" } };
  }
  if (request.method !== "POST") {
    return errorResponse("Use GET for the demo or POST a JSON compile request.", 405);
  }

  let body;
  try {
    body = await request.json();
  } catch {
    return errorResponse("Request body must be valid JSON.", 400);
  }
  if (body === null || typeof body !== "object" || Array.isArray(body)) {
    return errorResponse("Request body must be a JSON object.", 400);
  }
  if (typeof body.source !== "string" || body.source.length === 0) {
    return errorResponse("`source` must be a non-empty Vue SFC string.", 400);
  }
  if (new TextEncoder().encode(body.source).byteLength > MAX_SOURCE_BYTES) {
    return errorResponse("`source` must not exceed 1 MiB.", 413);
  }
  if (
    body.options !== undefined &&
    (body.options === null || typeof body.options !== "object" || Array.isArray(body.options))
  ) {
    return errorResponse("`options` must be a JSON object when provided.", 400);
  }

  return {
    source: body.source,
    options: { filename: "anonymous.vue", ...body.options },
  };
}

export async function handleRequest(request, getBinding) {
  const input = await inputFromRequest(request);
  if (input instanceof Response) {
    return input;
  }

  const vize = await getBinding();
  const result = vize.compileSfc(input.source, input.options);
  return Response.json({ ok: true, package: "@vizejs/wasm", result });
}
