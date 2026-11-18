// Options API matrix, part 3: cross-file shapes, compiler flags, consumers.
import { write } from "./cases1.ts";

const tsconfig = (extra: Record<string, unknown>, vue?: Record<string, unknown>) => JSON.stringify({
  compilerOptions: {
    target: "ESNext", module: "ESNext", moduleResolution: "bundler", strict: true, noEmit: true,
    skipLibCheck: true, jsx: "preserve", lib: ["ESNext", "DOM"], types: [], ...extra,
  },
  ...(vue ? { vueCompilerOptions: vue } : {}),
  include: ["src/**/*.ts", "src/**/*.vue", "src/**/*.js"],
  contentMappers: [{ package: "vize", extensions: [".vue"] }],
}, null, 2);

write("extends-from-sfc", {
  "src/Base.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { label: { type: String, required: true } },
  data() { return { baseCount: 0 } },
  computed: { baseUpper(): string { return this.label.toUpperCase() } },
  methods: { baseAdd(n: number) { this.baseCount += n } },
})
</script>

<template><div>{{ label }}</div></template>
`,
  "src/Derived.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import Base from './Base.vue'
export default defineComponent({
  extends: Base,
  data() { return { own: 1 } },
})
</script>

<template>
  <div>
    {{ label.toUpperCase() }} {{ baseCount.toFixed() }} {{ baseUpper.length }} {{ own }}
    <button @click="baseAdd(1)">ok</button>
    <button @click="baseAdd('x')">bad</button>
    {{ label.toFixed() }} {{ baseCount.toUpperCase() }} {{ nope }}
  </div>
</template>
`,
  "src/MixinUser.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import Base from './Base.vue'
export default defineComponent({
  mixins: [Base],
})
</script>

<template>
  <div>{{ baseCount.toFixed() }} <button @click="baseAdd('x')">bad</button> {{ baseUpper.toFixed() }}</div>
</template>
`,
});

write("ts-consumers-of-options-component", {
  "src/Child.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { title: { type: String, required: true }, size: { type: Number, default: 1 } },
  emits: { pick: (n: number) => true },
  data() { return { open: false } },
  methods: { toggle(force?: boolean) { this.open = force ?? !this.open; return this.open } },
})
</script>

<template><div>{{ title }}</div></template>
`,
  "src/use.ts": `
import { createApp, type ComponentPublicInstance } from 'vue'
import Child from './Child.vue'
type I = InstanceType<typeof Child>
declare const inst: I
const t: string = inst.title
const s: number = inst.size
const o: boolean = inst.toggle(true)
const bad: number = inst.title
inst.toggle('x')
inst.$emit('pick', 1)
inst.$emit('pick', 'x')
const p: I['$props'] = { title: 'a' }
const p2: I['$props'] = { title: 1 }
createApp(Child, { title: 'x' })
export const used: unknown[] = [t, s, o, bad, p, p2, inst as ComponentPublicInstance]
`,
  "src/SetupParent.vue": `
<script setup lang="ts">
import { useTemplateRef, onMounted } from 'vue'
import Child from './Child.vue'
const child = useTemplateRef<InstanceType<typeof Child>>('child')
onMounted(() => {
  child.value?.toggle()
  child.value?.toggle('x')
  const n: number = child.value!.title
  return n
})
</script>

<template>
  <Child ref="child" title="t" />
</template>
`,
});

write("strict-flags-rich", {
  "tsconfig.json": tsconfig({
    noUnusedLocals: true, noUnusedParameters: true, noImplicitReturns: true, noUncheckedIndexedAccess: true,
    exactOptionalPropertyTypes: true, noPropertyAccessFromIndexSignature: true, verbatimModuleSyntax: true,
    noImplicitOverride: true, noFallthroughCasesInSwitch: true,
  }),
  "src/Child.vue": `
<script lang="ts">
import { defineComponent, type PropType } from 'vue'
export default defineComponent({
  props: {
    title: { type: String, required: true },
    tags: { type: Array as PropType<string[]>, default: () => [] },
    opt: String,
  },
  emits: ['change'],
  data() { return { map: {} as Record<string, number>, n: 0 } },
  computed: { first(): string | undefined { return this.tags[0] } },
  methods: {
    handle(e: Event, unusedParam: number) { return e.type },
    unusedMethod() { return 1 },
  },
})
</script>

<template>
  <div @click="handle($event, 1)">
    {{ title }} {{ first?.length }} {{ tags[0].length }} {{ map.key }} {{ map['key'] }} {{ opt }}
  </div>
</template>
`,
  "src/Parent.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import Child from './Child.vue'
import UnusedChild from './Child.vue'
export default defineComponent({
  components: { Child },
  data() { return { maybe: undefined as string | undefined } },
})
</script>

<template>
  <Child title="x" :opt="maybe" @change="() => {}" />
</template>
`,
});

write("strict-templates-options", {
  "tsconfig.json": tsconfig({}, { strictTemplates: true }),
  "src/Child.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { title: { type: String, required: true } },
  emits: ['pick'],
})
</script>

<template><div>{{ title }}</div></template>
`,
  "src/Parent.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import Child from './Child.vue'
export default defineComponent({ components: { Child } })
</script>

<template>
  <Child title="a" @pick="() => {}" />
  <Child title="a" unknown-prop="x" @unknown-event="() => {}" />
  <NotRegistered />
</template>
`,
});

write("model-and-kebab-on-options-child", {
  "src/Field.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: {
    modelValue: { type: String, required: true },
    maxLength: { type: Number, default: 10 },
    count: { type: Number, default: 0 },
  },
  emits: {
    'update:modelValue': (v: string) => true,
    'update:count': (v: number) => true,
    itemClick: (id: number) => true,
  },
})
</script>

<template><input :value="modelValue" :maxlength="maxLength" /></template>
`,
  "src/Form.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import Field from './Field.vue'
export default defineComponent({
  components: { Field },
  data() { return { text: 'a', num: 1, total: 0 } },
  methods: { onItem(id: number) { return id }, onBadItem(id: string) { return id } },
})
</script>

<template>
  <Field v-model="text" :max-length="5" v-model:count="total" @item-click="onItem" />
  <Field v-model="num" />
  <Field v-model="text" :max-length="'5'" />
  <Field v-model="text" @item-click="onBadItem" />
  <Field v-model="text" v-bind="{ maxLength: 'x' }" />
  <component :is="Field" v-model="text" :max-length="'x'" />
</template>
`,
});

write("async-and-global-components", {
  "src/Child.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({ props: { title: { type: String, required: true } } })
</script>

<template><div>{{ title }}</div></template>
`,
  "src/globals.d.ts": `
import Child from './Child.vue'
declare module 'vue' {
  interface GlobalComponents { GlobalChild: typeof Child }
}
`,
  "src/Host.vue": `
<script lang="ts">
import { defineComponent, defineAsyncComponent } from 'vue'
export default defineComponent({
  components: { AsyncChild: defineAsyncComponent(() => import('./Child.vue')) },
})
</script>

<template>
  <AsyncChild title="a" />
  <AsyncChild :title="1" />
  <GlobalChild title="a" />
  <GlobalChild :title="1" />
  <global-child :title="2" />
</template>
`,
});

write("provide-inject-keys-and-setup", {
  "src/keys.ts": `
import type { InjectionKey } from 'vue'
export const themeKey: InjectionKey<{ dark: boolean }> = Symbol('theme')
`,
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent, inject, provide } from 'vue'
import { themeKey } from './keys'
export default defineComponent({
  setup() {
    provide(themeKey, { dark: true })
    const theme = inject(themeKey)!
    return { theme }
  },
  inject: { injected: { from: themeKey as symbol, default: () => ({ dark: false }) } },
})
</script>

<template><div>{{ theme.dark }} {{ theme.light }}</div></template>
`,
});

write("emits-array-template-payloads", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  emits: ['open', 'close'],
  methods: { fire() { this.$emit('open', 1, 2); this.$emit('nope') } },
})
</script>

<template>
  <div>
    <button @click="$emit('open', 1)">o</button>
    <button @click="$emit('close')">c</button>
    <button @click="$emit('other')">x</button>
  </div>
</template>
`,
});

write("plain-ts-emits-object-template", {
  "src/Comp.vue": `
<script lang="ts">
export default {
  emits: { save: (n: number) => true },
}
</script>

<template>
  <div>
    <button @click="$emit('save', 1)">ok</button>
    <button @click="$emit('save', 'x')">bad</button>
    <button @click="$emit('other')">other</button>
  </div>
</template>
`,
});

write("options-component-casing-registrations", {
  "src/Child.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({ props: { n: { type: Number, required: true } } })
</script>

<template><div>{{ n }}</div></template>
`,
  "src/Host.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import Child from './Child.vue'
export default defineComponent({
  components: { MyChild: Child, 'kebab-child': Child, lowerchild: Child },
})
</script>

<template>
  <MyChild :n="1" />
  <my-child :n="'a'" />
  <MyChild :n="'b'" />
  <kebab-child :n="'c'" />
  <KebabChild :n="'d'" />
  <lowerchild :n="'e'" />
  <Lowerchild :n="'f'" />
</template>
`,
});

write("js-jsdoc-methods-plain", {
  "tsconfig.json": tsconfig({ allowJs: true, checkJs: true }),
  "src/Comp.vue": `
<script>
/** @typedef {{ id: number, name: string }} Row */
export default {
  props: {
    /** @type {import('vue').PropType<Row[]>} */
    rows: { type: Array, required: true },
  },
  data() {
    return { /** @type {string | null} */ selected: null }
  },
  computed: {
    /** @returns {number} */
    total() { return this.rows.length },
  },
  methods: {
    /** @param {Row} row */
    pick(row) { this.selected = row.name },
    /**
     * @param {number} a
     * @param {number} b
     */
    add(a, b) { return a + b },
  },
}
</script>

<template>
  <div>
    <span v-for="r in rows" :key="r.id" @click="pick(r)">{{ r.name }}</span>
    {{ total.toFixed() }} {{ add(1, 2) }} {{ add('x', 2) }} {{ selected?.length }}
  </div>
</template>
`,
});

write("js-jsdoc-data-and-this", {
  "tsconfig.json": tsconfig({ allowJs: true, checkJs: true }),
  "src/Comp.vue": `
<script>
import { defineComponent } from 'vue'
export default defineComponent({
  data() {
    return {
      /** @type {string[]} */
      list: [],
      count: 0,
    }
  },
  methods: {
    /** @param {string} item */
    push(item) { this.list.push(item); this.count++ },
  },
})
</script>

<template>
  <div>
    {{ list.join() }} {{ count.toFixed() }}
    <button @click="push('a')">a</button>
    <button @click="push(1)">1</button>
  </div>
</template>
`,
});
