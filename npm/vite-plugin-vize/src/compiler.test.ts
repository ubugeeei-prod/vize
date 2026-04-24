import assert from "node:assert/strict";

import { compileBatch, compileFile } from "./compiler.ts";

const tresSource = `<script setup lang="ts">
import { Primitive } from "@tresjs/core";
const msg = "hello";
</script>

<template>
  <primitive />
  <div>{{ msg }}</div>
</template>`;

const clientCompiled = compileFile(
  "/src/TresPrimitive.vue",
  new Map(),
  { sourceMap: false, ssr: false, vapor: true },
  tresSource,
);

assert.match(
  clientCompiled.code,
  /const _component_primitive = _ctx\.Primitive/,
  "Client Vapor builds should resolve lowercase imported components through setup bindings",
);
assert.match(
  clientCompiled.code,
  /const __vaporRender = render/,
  "Client Vapor builds should preserve the render alias used by script setup output",
);

const ssrCompiled = compileFile(
  "/src/TresPrimitive.vue",
  new Map(),
  { sourceMap: false, ssr: true, vapor: true },
  tresSource,
);

assert.match(
  ssrCompiled.code,
  /\$setup\.Primitive/,
  "SSR builds should resolve lowercase imported components from setup bindings",
);
assert.doesNotMatch(
  ssrCompiled.code,
  /_resolveComponent\("primitive"\)/,
  "SSR builds must not fall back to runtime component resolution for imported lowercase components",
);
assert.match(
  ssrCompiled.code,
  /ssrRender/,
  "SSR builds should still emit server-renderer code paths when Vapor is enabled for the client",
);
assert.doesNotMatch(
  ssrCompiled.code,
  /__vapor/,
  "SSR builds should not keep the Vapor component marker when the compiler falls back to VDOM",
);

const batchResult = compileBatch(
  [
    {
      path: "/src/TresPrimitive.vue",
      source: tresSource,
    },
  ],
  new Map(),
  { ssr: true, vapor: true },
);

assert.equal(
  batchResult.successCount,
  1,
  "Batch compilation should succeed for the SSR regression",
);
assert.equal(
  batchResult.failedCount,
  0,
  "Batch compilation should stay clean for the SSR regression",
);
assert.equal(batchResult.results.length, 1, "Batch compilation should return a single file result");
assert.match(
  batchResult.results[0]?.code ?? "",
  /\$setup\.Primitive/,
  "Batch SSR compilation should match single-file binding resolution for lowercase imported components",
);
assert.doesNotMatch(
  batchResult.results[0]?.code ?? "",
  /__vapor/,
  "Batch SSR compilation should also drop the Vapor marker when falling back to VDOM",
);

const antDesignSource = `<script setup lang="ts">
import { Form, Input } from "ant-design-vue";
</script>

<template>
  <Form.Item label="Teacher">
    <Input />
  </Form.Item>
</template>`;

const antDesignDomCompiled = compileFile(
  "/src/AntDesignTeacher.vue",
  new Map(),
  { sourceMap: false, ssr: false, vapor: false },
  antDesignSource,
);

assert.match(
  antDesignDomCompiled.code,
  /_create(?:Block|VNode)\(Form\.Item/,
  "DOM builds should compile dotted component tags to direct setup member expressions",
);
assert.doesNotMatch(
  antDesignDomCompiled.code,
  /_resolveComponent\("Form\.Item"\)/,
  "DOM builds must not fall back to runtime resolution for dotted imported components",
);

const antDesignSsrCompiled = compileFile(
  "/src/AntDesignTeacher.vue",
  new Map(),
  { sourceMap: false, ssr: true, vapor: false },
  antDesignSource,
);

assert.match(
  antDesignSsrCompiled.code,
  /\$setup\.Form\.Item/,
  "SSR builds should preserve setup member expressions for dotted imported components",
);
assert.doesNotMatch(
  antDesignSsrCompiled.code,
  /_resolveComponent\("Form\.Item"\)/,
  "SSR builds must not fall back to runtime resolution for dotted imported components",
);

const antDesignVaporCompiled = compileFile(
  "/src/AntDesignTeacher.vue",
  new Map(),
  { sourceMap: false, ssr: false, vapor: true },
  antDesignSource,
);

assert.match(
  antDesignVaporCompiled.code,
  /const _component_Form_Item = _ctx\.Form\.Item/,
  "Vapor builds should resolve dotted imported components through ctx member expressions",
);
assert.doesNotMatch(
  antDesignVaporCompiled.code,
  /_resolveComponent\("Form\.Item"\)/,
  "Vapor builds must not leave dotted imported components to runtime resolution",
);

const customRendererSource = `<script setup lang="ts">
import { Primitive } from "@tresjs/core";
const visible = true;
</script>

<template>
  <mesh>
    <group v-if="visible">
      <primitive />
    </group>
  </mesh>
</template>`;

const customRendererClientCompiled = compileFile(
  "/src/TresCustomRenderer.vue",
  new Map(),
  { sourceMap: false, ssr: false, vapor: true, customRenderer: true },
  customRendererSource,
);

assert.match(
  customRendererClientCompiled.code,
  /const _component_primitive = _ctx\.Primitive/,
  "Custom renderer Vapor builds should still resolve imported lowercase components through setup bindings",
);
assert.doesNotMatch(
  customRendererClientCompiled.code,
  /_resolveComponent\("(mesh|group|primitive)"\)/,
  "Custom renderer Vapor builds must not fall back to runtime component resolution for intrinsic tags",
);

const customRendererSsrCompiled = compileFile(
  "/src/TresCustomRenderer.vue",
  new Map(),
  { sourceMap: false, ssr: true, vapor: true, customRenderer: true },
  customRendererSource,
);

assert.match(
  customRendererSsrCompiled.code,
  /\$setup\.Primitive/,
  "Custom renderer SSR builds should keep imported lowercase components bound to setup",
);
assert.doesNotMatch(
  customRendererSsrCompiled.code,
  /_resolveComponent\("(mesh|group|primitive)"\)/,
  "Custom renderer SSR builds must not resolve intrinsic tags as Vue components",
);
assert.doesNotMatch(
  customRendererSsrCompiled.code,
  /<primitive><\/primitive>/,
  "Custom renderer SSR builds must not stringify imported lowercase components as plain elements",
);
assert.doesNotMatch(
  customRendererSsrCompiled.code,
  /__vapor/,
  "Custom renderer SSR builds should not keep the Vapor marker after the SSR fallback",
);

const ssrSuspenseSource = `<script setup lang="ts">
let visible = true;
</script>

<template>
  <Suspense>
    <div v-if="visible">ready</div>
  </Suspense>
</template>`;

const ssrSuspenseCompiled = compileFile(
  "/src/SsrSuspense.vue",
  new Map(),
  { sourceMap: false, ssr: true, vapor: false },
  ssrSuspenseSource,
);

assert.match(
  ssrSuspenseCompiled.code,
  /ssrRenderSuspense as _ssrRenderSuspense/,
  "SSR builds should render built-in Suspense with the dedicated server-renderer helper",
);
assert.doesNotMatch(
  ssrSuspenseCompiled.code,
  /_resolveComponent\("Suspense"\)/,
  "SSR builds must not resolve built-in Suspense through runtime component lookup",
);
assert.match(
  ssrSuspenseCompiled.code,
  /unref as _unref/,
  "SSR builds should import unref when transformed setup bindings use _unref()",
);

const ssrComponentPropsSource = `<script setup lang="ts">
const ErrorBoundary = {};
let error = { message: "boom" };
</script>

<template>
  <Suspense>
    <ErrorBoundary :error="error" />
    <Transition>
      <I18nT keypath="build.environment" tag="span">node</I18nT>
    </Transition>
  </Suspense>
</template>`;

const ssrComponentPropsCompiled = compileFile(
  "/src/SsrComponentProps.vue",
  new Map(),
  { sourceMap: false, ssr: true, vapor: false },
  ssrComponentPropsSource,
);

assert.match(
  ssrComponentPropsCompiled.code,
  /\$setup\.ErrorBoundary, \{ error: _unref\(\$setup\.error\) \}/,
  "SSR builds should pass dynamic props to setup-bound components",
);
assert.match(
  ssrComponentPropsCompiled.code,
  /_resolveComponent\("I18nT"\), \{\s*keypath: "build\.environment",\s*tag: "span"\s*\}/,
  "SSR builds should pass static props to runtime-resolved components",
);
assert.doesNotMatch(
  ssrComponentPropsCompiled.code,
  /_resolveComponent\("Transition"\)/,
  "SSR builds must not resolve built-in Transition through runtime component lookup",
);
assert.match(
  ssrComponentPropsCompiled.code,
  /return \[_createTextVNode\("node"\)\]/,
  "SSR component slots should return VNodes when invoked without the server _push callback",
);

const ssrTeleportSource = `<template>
  <Teleport to="body">
    <Transition>
      <div v-if="visible">tip</div>
    </Transition>
  </Teleport>
</template>`;

const ssrTeleportCompiled = compileFile(
  "/src/SsrTeleport.vue",
  new Map(),
  { sourceMap: false, ssr: true, vapor: false },
  ssrTeleportSource,
);

assert.match(
  ssrTeleportCompiled.code,
  /ssrRenderTeleport as _ssrRenderTeleport/,
  "SSR builds should render built-in Teleport with the dedicated server-renderer helper",
);
assert.doesNotMatch(
  ssrTeleportCompiled.code,
  /_resolveComponent\("(Teleport|Transition)"\)/,
  "SSR builds must not resolve built-in Teleport or nested Transition through runtime lookup",
);

const ssrSlotForSource = `<template>
  <Foo>
    <NuxtLink v-for="[dep, version] in deps" :key="dep" :to="packageRoute(dep, version)">
      {{ dep }}
    </NuxtLink>
  </Foo>
</template>`;

const ssrSlotForCompiled = compileFile(
  "/src/SsrSlotFor.vue",
  new Map(),
  { sourceMap: false, ssr: true, vapor: false },
  ssrSlotForSource,
);

assert.match(
  ssrSlotForCompiled.code,
  /_ssrRenderList\(_ctx\.deps, \(\[dep, version\]\) =>/,
  "SSR component slots should preserve destructured v-for aliases as local callback params",
);
assert.match(
  ssrSlotForCompiled.code,
  /to: _ctx\.packageRoute\(dep, version\)/,
  "SSR component slots should not prefix v-for aliases with _ctx inside child component props",
);
assert.doesNotMatch(
  ssrSlotForCompiled.code,
  /_ctx\.(dep|version)\b/,
  "SSR component slots should not leak destructured v-for aliases to instance context",
);

const ssrScopedSlotSource = `<script setup lang="ts">
import { valibotResolver } from "@primevue/forms/resolvers/valibot";
const schema = {};
</script>

<template>
  <PForm :resolver="schema ? valibotResolver(schema) : undefined">
    <template #item="{ data: speaker }">
      <div :style="{ backgroundImage: \`url(\${speaker.avatarUrl})\` }">{{ speaker.name }}</div>
    </template>
  </PForm>
</template>`;

const ssrScopedSlotCompiled = compileFile(
  "/src/SsrScopedSlot.vue",
  new Map(),
  { sourceMap: false, ssr: true, vapor: false },
  ssrScopedSlotSource,
);

assert.match(
  ssrScopedSlotCompiled.code,
  /return \{\s*valibotResolver,\s*schema\s*\}/,
  "SSR script setup should return imports that are only used inside template expressions",
);
assert.match(
  ssrScopedSlotCompiled.code,
  /resolver: \$setup\.schema \? (?:_unref\(\$setup\.valibotResolver\)|\$setup\.valibotResolver)\(\$setup\.schema\) : undefined/,
  "SSR component props should call template-only imports through setup bindings",
);
assert.match(
  ssrScopedSlotCompiled.code,
  /item: _withCtx\(\(\{ data: speaker \}, _push, _parent, _scopeId\) =>/,
  "SSR component slots should preserve destructured slot prop parameters",
);
assert.doesNotMatch(
  ssrScopedSlotCompiled.code,
  /_ctx\.speaker\b/,
  "SSR scoped slot bodies should not read slot props from instance context",
);

const ssrNormalScriptImportSource = `<script lang="ts">
import {
  type FormFieldState,
  Form as PForm,
} from "@primevue/forms";
import { valibotResolver } from "@primevue/forms/resolvers/valibot";

export interface FormProps {
  schema?: unknown;
}
</script>

<script setup lang="ts">
const { schema } = defineProps<FormProps>();
const emit = defineEmits<{ submit: [] }>();
</script>

<template>
  <PForm :resolver="schema ? valibotResolver(schema) : undefined" @submit="emit('submit')" />
</template>`;

const ssrNormalScriptImportCompiled = compileFile(
  "/src/SsrNormalScriptImport.vue",
  new Map(),
  { sourceMap: false, ssr: true, vapor: false },
  ssrNormalScriptImportSource,
);

assert.match(
  ssrNormalScriptImportCompiled.code,
  /return \{\s*emit,\s*PForm,\s*valibotResolver\s*\}/,
  "SSR script setup should return normal-script imports that are used by template expressions",
);
assert.match(
  ssrNormalScriptImportCompiled.code,
  /import \{ Form as PForm \} from ["']@primevue\/forms["']/,
  "SSR output should preserve value imports used only through setup bindings",
);
assert.match(
  ssrNormalScriptImportCompiled.code,
  /resolver: \$props\.schema \? (?:_unref\(\$setup\.valibotResolver\)|\$setup\.valibotResolver)\(\$props\.schema\) : undefined/,
  "SSR component props should call normal-script template imports through setup bindings",
);

console.log("✅ vite-plugin-vize compiler tests passed!");
