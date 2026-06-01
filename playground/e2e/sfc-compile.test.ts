import { describe, it, expect, beforeAll } from "vite-plus/test";
import { loadWasm, type WasmModule } from "../src/wasm/index";

describe("SFC Compilation", () => {
  let wasm: WasmModule | null = null;

  beforeAll(async () => {
    wasm = await loadWasm();
  });

  describe("Basic SFC", () => {
    it("should compile template-only SFC", () => {
      const sfc = `
<template>
  <div>Hello World</div>
</template>
`;
      const result = wasm!.compileSfc(sfc, {});
      expect(result).toBeDefined();
      expect(result.descriptor).toBeDefined();
      expect(result.descriptor.template).toBeDefined();
      expect(result.descriptor.template?.content).toContain("Hello World");
    });

    it("should compile SFC with script setup", () => {
      const sfc = `
<script setup>
const msg = 'Hello'
</script>

<template>
  <div>{{ msg }}</div>
</template>
`;
      const result = wasm!.compileSfc(sfc, {});
      expect(result).toBeDefined();
      expect(result.descriptor.scriptSetup).toBeDefined();
      expect(result.script?.code).toBeDefined();
    });

    it("should compile SFC with both script and script setup", () => {
      const sfc = `
<script>
export default {
  name: 'MyComponent'
}
</script>

<script setup>
const count = 0
</script>

<template>
  <div>{{ count }}</div>
</template>
`;
      const result = wasm!.compileSfc(sfc, {});
      expect(result.descriptor.script).toBeDefined();
      expect(result.descriptor.scriptSetup).toBeDefined();
    });
  });

  describe("TypeScript Support", () => {
    it("should strip type annotations from script setup", () => {
      const sfc = `
<script setup lang="ts">
const count: number = 0
const name: string = 'test'
</script>

<template>
  <div>{{ count }}</div>
</template>
`;
      const result = wasm!.compileSfc(sfc, {});
      // Check code is defined (may be in script.code or result directly)
      const code = result.script?.code;
      expect(code).toBeDefined();
      // Type annotations should be stripped from the output
      // The compiled output should have 'const count = 0' not 'const count: number = 0'
      expect(code).toContain("count");
      expect(code).toContain("name");
      expect(code).not.toContain(": number");
      expect(code).not.toContain(": string");
    });

    it("should strip generic type parameters from ref/reactive", () => {
      const sfc = `
<script setup lang="ts">
import { ref, reactive } from 'vue'
const count = ref<number>(0)
const items = ref<string[]>([])
const state = reactive<{ name: string }>({ name: 'test' })
</script>

<template>
  <div>{{ count }}</div>
</template>
`;
      const result = wasm!.compileSfc(sfc, {});
      const code = result.script?.code;
      expect(code).toBeDefined();
      // The compiled code should contain the variables
      expect(code).toContain("count");
      expect(code).toContain("items");
      expect(code).toContain("state");
      expect(code).not.toContain("ref<number>");
      expect(code).not.toContain("ref<string[]>");
      expect(code).not.toContain("reactive<{ name: string }>");
    });

    it("should handle complex generic types", () => {
      const sfc = `
<script setup lang="ts">
import { ref, shallowRef } from 'vue'
const editorRef = ref<HTMLDivElement | null>(null)
const instance = shallowRef<Map<string, number> | null>(null)
</script>

<template>
  <div ref="editorRef"></div>
</template>
`;
      const result = wasm!.compileSfc(sfc, {});
      const code = result.script?.code;
      expect(code).toBeDefined();
      // The compiled code should contain the variables
      expect(code).toContain("editorRef");
      expect(code).toContain("instance");
      expect(code).not.toContain("ref<HTMLDivElement | null>");
      expect(code).not.toContain("shallowRef<Map<string, number> | null>");
    });

    it("should handle interface declarations", () => {
      const sfc = `
<script setup lang="ts">
interface User {
  name: string
  age: number
}
const user: User = { name: 'test', age: 25 }
</script>

<template>
  <div>{{ user.name }}</div>
</template>
`;
      const result = wasm!.compileSfc(sfc, { scriptExt: "preserve" });
      expect(result.script?.code).toBeDefined();
      // TypeScript should remain available when preserve mode is requested.
      expect(result.script?.code).toContain("interface User");
    });

    it("should handle type aliases", () => {
      const sfc = `
<script setup lang="ts">
type Status = 'active' | 'inactive'
const status: Status = 'active'
</script>

<template>
  <div>{{ status }}</div>
</template>
`;
      const result = wasm!.compileSfc(sfc, { scriptExt: "preserve" });
      expect(result.script?.code).toBeDefined();
      // TypeScript should remain available when preserve mode is requested.
      expect(result.script?.code).toContain("type Status");
    });
  });

  describe("Props and Emits", () => {
    it("should handle defineProps with type parameter", () => {
      const sfc = `
<script setup lang="ts">
defineProps<{
  title: string
  count?: number
}>()
</script>

<template>
  <div>{{ title }}</div>
</template>
`;
      const result = wasm!.compileSfc(sfc, {});
      expect(result.script?.code).toBeDefined();
      expect(result.script?.bindings).toBeDefined();
    });

    it("should handle defineEmits with type parameter", () => {
      const sfc = `
<script setup lang="ts">
const emit = defineEmits<{
  (e: 'update', value: number): void
  (e: 'close'): void
}>()
</script>

<template>
  <button @click="emit('close')">Close</button>
</template>
`;
      const result = wasm!.compileSfc(sfc, {});
      expect(result.script?.code).toBeDefined();
    });

    it("should handle withDefaults", () => {
      const sfc = `
<script setup lang="ts">
interface Props {
  msg?: string
  count?: number
}
const props = withDefaults(defineProps<Props>(), {
  msg: 'hello',
  count: 0
})
</script>

<template>
  <div>{{ props.msg }}</div>
</template>
`;
      const result = wasm!.compileSfc(sfc, {});
      expect(result.script?.code).toBeDefined();
    });
  });

  describe("Template Compilation", () => {
    it("should compile v-if directive", () => {
      const sfc = `
<script setup>
const show = true
</script>

<template>
  <div v-if="show">Visible</div>
</template>
`;
      const result = wasm!.compileSfc(sfc, {});
      expect(result.template?.code).toBeDefined();
    });

    it("should compile v-for directive", () => {
      const sfc = `
<script setup>
const items = [1, 2, 3]
</script>

<template>
  <div v-for="item in items" :key="item">{{ item }}</div>
</template>
`;
      const result = wasm!.compileSfc(sfc, {});
      expect(result.template?.code).toBeDefined();
    });

    it("should compile v-model directive", () => {
      const sfc = `
<script setup>
import { ref } from 'vue'
const text = ref('')
</script>

<template>
  <input v-model="text" />
</template>
`;
      const result = wasm!.compileSfc(sfc, {});
      expect(result.template?.code).toBeDefined();
    });

    it("should compile event handlers", () => {
      const sfc = `
<script setup>
function handleClick() {}
</script>

<template>
  <button @click="handleClick">Click</button>
</template>
`;
      const result = wasm!.compileSfc(sfc, {});
      expect(result.template?.code).toBeDefined();
    });

    it("should compile slot content", () => {
      const sfc = `
<script setup>
</script>

<template>
  <div>
    <slot name="header"></slot>
    <slot></slot>
    <slot name="footer"></slot>
  </div>
</template>
`;
      const result = wasm!.compileSfc(sfc, {});
      expect(result.template?.code).toBeDefined();
    });
  });

  describe("SSR Mode", () => {
    it("should compile SFC in SSR mode", () => {
      const sfc = `
<script setup>
const msg = 'Hello SSR'
</script>

<template>
  <div>{{ msg }}</div>
</template>
`;
      const result = wasm!.compileSfc(sfc, { ssr: true });
      expect(result).toBeDefined();
      expect(result.script?.code).toBeDefined();
    });

    it("should resolve kebab-case imported components through setup bindings in both DOM and SSR output", () => {
      const sfc = `
<script setup lang="ts">
import DashTest from './dash-test.vue'
</script>

<template>
  <dash-test />
  <DashTest />
</template>
`;
      const domResult = wasm!.compileSfc(sfc, {});
      expect(domResult.script?.code).toContain("_createVNode(DashTest)");
      expect(domResult.script?.code).not.toContain('_resolveComponent("dash-test")');

      const ssrResult = wasm!.compileSfc(sfc, { ssr: true });
      expect(ssrResult.script?.code).toContain("$setup.DashTest");
      expect(ssrResult.script?.code).not.toContain('_resolveComponent("dash-test")');
    });

    it("should resolve dotted component tags through setup member bindings in both DOM and SSR output", () => {
      const sfc = `
<script setup lang="ts">
import { Form, Input } from 'ant-design-vue'
</script>

<template>
  <Form.Item label="Teacher">
    <Input />
      </Form.Item>
</template>
`;
      const domResult = wasm!.compileSfc(sfc, {});
      expect(domResult.script?.code).toMatch(/_create(?:Block|VNode)\(Form\.Item/);
      expect(domResult.script?.code).not.toContain('_resolveComponent("Form.Item")');

      const ssrResult = wasm!.compileSfc(sfc, { ssr: true });
      expect(ssrResult.script?.code).toContain("$setup.Form.Item");
      expect(ssrResult.script?.code).not.toContain('_resolveComponent("Form.Item")');
    });

    it("should resolve lowercase imported components from setup bindings in SSR mode", () => {
      const sfc = `
<script setup lang="ts">
import { Primitive } from '@tresjs/core'
</script>

<template>
  <primitive />
</template>
`;
      const result = wasm!.compileSfc(sfc, { ssr: true });
      expect(result.script?.code).toContain("_ssrRenderComponent($setup.Primitive");
      expect(result.script?.code).not.toContain('_resolveComponent("primitive")');
    });

    it("should keep custom renderer intrinsics as elements while preserving imported lowercase bindings", () => {
      const sfc = `
<script setup lang="ts">
import { Primitive } from '@tresjs/core'
const visible = true
</script>

<template>
  <mesh>
    <group v-if="visible">
      <primitive />
    </group>
  </mesh>
</template>
`;
      const domResult = wasm!.compileSfc(sfc, { customRenderer: true });
      expect(domResult.script?.code).toContain("Primitive");
      expect(domResult.script?.code).not.toContain('_createElementBlock("primitive")');
      expect(domResult.script?.code).not.toContain('_resolveComponent("group")');

      const ssrResult = wasm!.compileSfc(sfc, {
        ssr: true,
        customRenderer: true,
      });
      expect(ssrResult.script?.code).toContain("$setup.Primitive");
      expect(ssrResult.script?.code).not.toContain('_resolveComponent("group")');
      expect(ssrResult.script?.code).not.toContain("<primitive></primitive>");
    });
  });
});
