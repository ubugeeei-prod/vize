// Options API matrix, part 1: defineComponent / plain object in lang="ts".
import * as fs from "node:fs";
import * as path from "node:path";

const S = path.dirname(new URL(import.meta.url).pathname);
export function write(id: string, files: Record<string, string>) {
  const dir = path.join(S, "matrix", id);
  fs.rmSync(dir, { recursive: true, force: true });
  for (const [f, c] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(dir, f)), { recursive: true });
    fs.writeFileSync(path.join(dir, f), c.replace(/^\n/, ""));
  }
}

const RICH_SCRIPT = (wrap: (o: string) => string, lang = ` lang="ts"`) => `
<script${lang}>
import { defineComponent, type PropType } from 'vue'
interface Item { id: number; name: string }
export default ${wrap(`{
  props: {
    title: { type: String, required: true },
    count: { type: Number, default: 0 },
    items: { type: Array as PropType<Item[]>, default: () => [] },
    mode: { type: String as PropType<'a' | 'b'>, validator: (v: string) => ['a', 'b'].includes(v) },
    flag: Boolean,
    loose: null,
    multi: [String, Number],
    maybe: String,
  },
  emits: ['change'],
  data() {
    return { local: 1, list: [] as string[] }
  },
  computed: {
    upper(): string { return this.title.toUpperCase() },
    rw: {
      get(): number { return this.count },
      set(v: number) { this.local = v },
    },
  },
  methods: {
    inc(by: number) { this.local += by; this.$emit('change') },
  },
  watch: {
    count(n: number) { this.local = n },
  },
}`)}
</script>
`;

const dc = (o: string) => `defineComponent(${o})`;
const plain = (o: string) => o;

const CLEAN_TEMPLATE = `
<template>
  <div :title="title.toUpperCase()">
    {{ count.toFixed(2) }} {{ items.map((i) => i.name).join() }} {{ mode }}
    {{ flag ? 'y' : 'n' }} {{ upper.length }} {{ rw.toFixed() }} {{ local + 1 }}
    {{ list.length }} {{ multi }} {{ loose }} {{ maybe?.length }}
    <button @click="inc(1)">+</button>
    <input v-model="rw" type="number" />
    <span v-for="item in items" :key="item.id">{{ item.name.toUpperCase() }}</span>
  </div>
</template>
`;

const ERROR_TEMPLATE = `
<template>
  <div>
    {{ title.toFixed() }}
    {{ count.toUpperCase() }}
    {{ unknownName }}
    <button @click="inc('x')">+</button>
    {{ items[0]!.nope }}
    {{ upper.toFixed() }}
    {{ local.length }}
    {{ maybe.length }}
    {{ mode === 'c' }}
  </div>
</template>
`;

for (const [name, wrap] of [["dc", dc], ["plain", plain]] as const) {
  write(`${name}-rich-clean`, { "src/Rich.vue": RICH_SCRIPT(wrap) + CLEAN_TEMPLATE });
  write(`${name}-rich-template-errors`, { "src/Rich.vue": RICH_SCRIPT(wrap) + ERROR_TEMPLATE });
}

write("dc-script-errors", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { title: { type: String, required: true } },
  emits: ['change'],
  data() { return { n: 1 } },
  methods: {
    run() {
      this.nope()
      this.title = 'x'
      this.$emit('unknown')
      const s: string = this.n
      return s
    },
  },
})
</script>

<template>
  <div>{{ title }}</div>
</template>
`,
});

write("dc-emits-object", {
  "src/Child.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  emits: {
    save: (payload: { id: number }) => payload.id > 0,
    close: null,
    'update:modelValue': (value: string) => typeof value === 'string',
  },
  props: { modelValue: { type: String, default: '' } },
})
</script>

<template>
  <div>
    <button @click="$emit('save', { id: 1 })">ok</button>
    <button @click="$emit('save', { id: 'x' })">bad</button>
    <button @click="$emit('close')">close</button>
    <button @click="$emit('missing')">missing</button>
    <input :value="modelValue" @input="$emit('update:modelValue', 1)" />
  </div>
</template>
`,
  "src/Parent.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import Child from './Child.vue'
export default defineComponent({
  components: { Child },
  data() { return { text: 'a', num: 1 } },
  methods: {
    onSave(p: { id: number }) { return p.id },
    onBadSave(p: { id: string }) { return p.id },
  },
})
</script>

<template>
  <Child @save="onSave" @close="() => {}" v-model="text" />
  <Child @save="onBadSave" />
  <Child v-model="num" />
</template>
`,
});

write("dc-child-props-from-options-parent", {
  "src/Child.vue": `
<script lang="ts">
import { defineComponent, type PropType } from 'vue'
export default defineComponent({
  props: {
    title: { type: String, required: true },
    size: { type: Number, default: 1 },
    kind: { type: String as PropType<'a' | 'b'>, default: 'a' },
    tags: { type: Array as PropType<string[]> },
  },
  emits: { pick: (n: number) => true },
})
</script>

<template><div>{{ title }}{{ size }}{{ kind }}{{ tags }}</div></template>
`,
  "src/Parent.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import Child from './Child.vue'
export default defineComponent({
  components: { Child },
  data() { return { n: 1, s: 'x' } },
  methods: { onPick(n: number) { this.n = n } },
})
</script>

<template>
  <Child title="ok" :size="n" kind="b" :tags="[s]" @pick="onPick" />
  <Child :title="n" />
  <Child />
  <Child title="x" kind="c" />
  <Child title="x" :tags="[n]" />
  <Child title="x" @pick="(v: string) => v" />
</template>
`,
});

write("dc-child-props-from-setup-parent", {
  "src/Child.vue": `
<script lang="ts">
import { defineComponent, type PropType } from 'vue'
export default defineComponent({
  props: {
    title: { type: String, required: true },
    size: { type: Number, default: 1 },
    kind: { type: String as PropType<'a' | 'b'>, default: 'a' },
  },
  emits: { pick: (n: number) => true },
})
</script>

<template><div>{{ title }}{{ size }}{{ kind }}</div></template>
`,
  "src/Parent.vue": `
<script setup lang="ts">
import { ref } from 'vue'
import Child from './Child.vue'
const n = ref(1)
function onPick(v: number) { n.value = v }
</script>

<template>
  <Child title="ok" :size="n" @pick="onPick" />
  <Child :title="n" />
  <Child />
  <Child title="x" kind="c" />
  <Child title="x" @pick="(v: string) => v" />
</template>
`,
});

write("plain-child-props-from-setup-parent", {
  "src/Child.vue": `
<script lang="ts">
export default {
  props: {
    title: { type: String, required: true },
    size: { type: Number, default: 1 },
  },
  emits: ['pick'],
}
</script>

<template><div>{{ title }}{{ size }}</div></template>
`,
  "src/Parent.vue": `
<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child title="ok" :size="2" @pick="() => {}" />
  <Child :title="1" />
  <Child />
</template>
`,
});

write("dc-inject-provide", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  inject: {
    theme: { from: 'theme', default: 'light' },
    api: 'api',
  },
  provide() {
    return { depth: 1 }
  },
  data() { return { x: 1 } },
})
</script>

<template>
  <div>{{ theme }} {{ api }} {{ x }} {{ missingInjected }}</div>
</template>
`,
  "src/Arr.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  inject: ['foo', 'bar'],
})
</script>

<template>
  <div>{{ foo }} {{ bar }} {{ baz }}</div>
</template>
`,
});

write("dc-mixins-extends", {
  "src/mixins.ts": `
import { defineComponent } from 'vue'
export const counter = defineComponent({
  data() { return { count: 0 } },
  computed: { double(): number { return this.count * 2 } },
  methods: { increment() { this.count++ } },
})
export const named = defineComponent({
  props: { label: { type: String, required: true } },
  methods: { greet(who: string) { return this.label + who } },
})
export const base = {
  data() { return { baseValue: 'b' } },
  methods: { baseMethod(n: number) { return n } },
}
`,
  "src/Mixed.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import { counter, named, base } from './mixins'
export default defineComponent({
  mixins: [counter, named],
  extends: base,
  data() { return { own: true } },
})
</script>

<template>
  <div>
    {{ count.toFixed() }} {{ double }} {{ label.toUpperCase() }} {{ own }}
    <button @click="increment">+</button>
    {{ greet('x') }} {{ baseValue.toUpperCase() }} {{ baseMethod(1) }}
  </div>
</template>
`,
  "src/MixedErrors.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import { counter, named, base } from './mixins'
export default defineComponent({
  mixins: [counter, named],
  extends: base,
})
</script>

<template>
  <div>
    {{ count.toUpperCase() }} {{ greet(1) }} {{ label.toFixed() }}
    {{ baseValue.toFixed() }} {{ baseMethod('x') }} {{ notInMixin }}
  </div>
</template>
`,
});

write("dc-components-directives", {
  "src/Child.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({ props: { title: { type: String, required: true } } })
</script>

<template><div>{{ title }}</div></template>
`,
  "src/Host.vue": `
<script lang="ts">
import { defineComponent, type Directive } from 'vue'
import Child from './Child.vue'
const vColor: Directive<HTMLElement, string> = { mounted(el, binding) { el.style.color = binding.value } }
export default defineComponent({
  components: { Child, Renamed: Child },
  directives: {
    focus: { mounted(el: HTMLElement) { el.focus() } },
    color: vColor,
  },
  data() { return { c: 'red', n: 1 } },
})
</script>

<template>
  <Child title="a" />
  <Renamed title="b" />
  <renamed :title="1" />
  <input v-focus v-color="c" />
  <input v-color="n" />
</template>
`,
});

write("dc-expose-and-refs", {
  "src/Child.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  expose: ['reset'],
  data() { return { v: 1 } },
  methods: { reset() { this.v = 0 }, hidden() { return 1 } },
})
</script>

<template><div ref="root">{{ v }}</div></template>
`,
  "src/Parent.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import Child from './Child.vue'
export default defineComponent({
  components: { Child },
  methods: {
    go() {
      const c = this.$refs.child as InstanceType<typeof Child>
      c.reset()
    },
  },
})
</script>

<template>
  <Child ref="child" />
  <input ref="input" />
  <button @click="go">go</button>
  {{ $refs.input }}
</template>
`,
});

write("dc-setup-plus-options", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent, ref, computed } from 'vue'
export default defineComponent({
  props: { start: { type: Number, default: 0 } },
  setup(props) {
    const n = ref(props.start)
    const label = computed(() => String(n.value))
    function bump(by: number) { n.value += by }
    return { n, label, bump }
  },
  data() { return { extra: 'e' } },
  computed: { both(): string { return this.label + this.extra } },
  methods: { twice() { this.bump(2); return this.n * 2 } },
})
</script>

<template>
  <div>
    {{ n.toFixed() }} {{ label.toUpperCase() }} {{ both.length }} {{ twice() }} {{ start }}
    <button @click="bump(1)">+</button>
  </div>
</template>
`,
  "src/CompErrors.vue": `
<script lang="ts">
import { defineComponent, ref } from 'vue'
export default defineComponent({
  setup() {
    const n = ref(1)
    return { n, bump: (by: number) => { n.value += by } }
  },
  data() { return { extra: 'e' } },
})
</script>

<template>
  <div>
    {{ n.toUpperCase() }} {{ extra.toFixed() }}
    <button @click="bump('x')">+</button>
  </div>
</template>
`,
});

write("dc-instance-dollar-members", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  inheritAttrs: false,
  props: { title: { type: String, required: true } },
  emits: { pick: (n: number) => true },
})
</script>

<template>
  <div v-bind="$attrs">
    {{ $props.title.toUpperCase() }}
    {{ $attrs.class }}
    <slot v-if="$slots.default" />
    {{ $el }}
    {{ $refs }}
    {{ $options.name }}
    {{ $parent }} {{ $root }} {{ $data }}
    <button @click="$emit('pick', 1)">p</button>
    <button @click="$emit('pick', 'x')">p</button>
    {{ $props.nope }}
    {{ $nextTick }} {{ $forceUpdate }} {{ $watch }}
  </div>
</template>
`,
});

write("dc-global-properties", {
  "src/globals.d.ts": `
export {}
declare module 'vue' {
  interface ComponentCustomProperties {
    $t: (key: string) => string
    $filters: { money(n: number): string }
  }
}
`,
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  data() { return { amount: 1 } },
  methods: { label() { return this.$t('x') + this.$filters.money(this.amount) } },
})
</script>

<template>
  <div>
    {{ $t('hello') }} {{ $filters.money(amount) }}
    {{ $t(1) }} {{ $filters.money('x') }} {{ $missingGlobal }}
  </div>
</template>
`,
});

write("dc-vue-router", {
  "src/router.ts": `
import { createRouter, createWebHistory } from 'vue-router'
export const router = createRouter({ history: createWebHistory(), routes: [] })
`,
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  computed: { id(): string { return String(this.$route.params.id) } },
  methods: { go() { this.$router.push('/') } },
})
</script>

<template>
  <div>
    {{ $route.path.toUpperCase() }} {{ id }} {{ $route.query.q }}
    <button @click="$router.push({ name: 'home' })">home</button>
    {{ $route.nope }} {{ $router.nope() }}
  </div>
</template>
`,
});

write("dc-pinia-map-helpers", {
  "src/store.ts": `
import { defineStore } from 'pinia'
export const useCounter = defineStore('counter', {
  state: () => ({ count: 0, name: 'c' }),
  getters: { double: (s) => s.count * 2 },
  actions: { increment(by: number) { this.count += by } },
})
`,
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import { mapState, mapActions, mapWritableState, mapStores } from 'pinia'
import { useCounter } from './store'
export default defineComponent({
  computed: {
    ...mapState(useCounter, ['count', 'double']),
    ...mapState(useCounter, { renamed: 'name', fn: (s) => s.count + 1 }),
    ...mapWritableState(useCounter, ['name']),
    ...mapStores(useCounter),
  },
  methods: { ...mapActions(useCounter, ['increment']) },
})
</script>

<template>
  <div>
    {{ count.toFixed() }} {{ double.toFixed() }} {{ renamed.toUpperCase() }} {{ fn.toFixed() }}
    {{ name }} {{ counterStore.count }}
    <button @click="increment(1)">+</button>
    <input v-model="name" />
  </div>
</template>
`,
  "src/CompErrors.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import { mapState, mapActions } from 'pinia'
import { useCounter } from './store'
export default defineComponent({
  computed: { ...mapState(useCounter, ['count']) },
  methods: { ...mapActions(useCounter, ['increment']) },
})
</script>

<template>
  <div>
    {{ count.toUpperCase() }}
    <button @click="increment('x')">+</button>
    {{ double }}
  </div>
</template>
`,
});

write("dc-vuex-map-helpers", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import { mapGetters, mapState, mapActions, mapMutations } from 'vuex'
interface State { count: number; user: { name: string } }
export default defineComponent({
  computed: {
    ...mapGetters(['total', 'isReady']),
    ...mapState({ count: (s: any) => (s as State).count }),
    ...mapState(['user']),
  },
  methods: { ...mapActions(['load']), ...mapMutations({ setCount: 'SET_COUNT' }) },
})
</script>

<template>
  <div>
    {{ total.anything }} {{ isReady }} {{ count }} {{ user.name }}
    <button @click="load(); setCount(1)">x</button>
  </div>
</template>
`,
});

write("dc-watch-variants", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  data() { return { a: { b: 1 }, list: [1, 2] } },
  watch: {
    'a.b': { handler: 'onChange', deep: true, immediate: true },
    list: [{ handler(v: number[]) { return v.length } }, 'onChange'],
    a(v: { b: number }) { this.onChange(v.b) },
  },
  methods: { onChange(v: number) { return v } },
})
</script>

<template><div>{{ a.b }} {{ list.length }}</div></template>
`,
});

write("dc-template-assignments", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { count: { type: Number, required: true } },
  data() { return { local: 0, text: '' } },
  computed: { ro(): number { return this.local } },
})
</script>

<template>
  <div>
    <button @click="local++">ok</button>
    <button @click="count++">prop</button>
    <button @click="ro = 2">computed</button>
    <button @click="text = 1">type</button>
    <input v-model="count" />
  </div>
</template>
`,
});

write("dc-slots-type", {
  "src/List.vue": `
<script lang="ts">
import { defineComponent, type PropType, type SlotsType } from 'vue'
export default defineComponent({
  props: { items: { type: Array as PropType<string[]>, required: true } },
  slots: Object as SlotsType<{ default: { item: string; index: number }; empty: {} }>,
})
</script>

<template>
  <ul>
    <li v-for="(item, index) in items" :key="index"><slot :item="item" :index="index" /></li>
    <slot v-if="!items.length" name="empty" />
  </ul>
</template>
`,
  "src/Use.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import List from './List.vue'
export default defineComponent({ components: { List } })
</script>

<template>
  <List :items="['a']" v-slot="{ item, index }">{{ item.toUpperCase() }} {{ index.toFixed() }}</List>
  <List :items="['a']" v-slot="{ item }">{{ item.toFixed() }}</List>
  <List :items="[1]" />
</template>
`,
});

write("dc-required-optional-props-strict", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent, type PropType } from 'vue'
export default defineComponent({
  props: {
    optional: String,
    withDefault: { type: String, default: 'x' },
    obj: { type: Object as PropType<{ a: number }> },
    req: { type: Object as PropType<{ a: number }>, required: true },
    fn: { type: Function as PropType<(n: number) => string>, required: true },
    date: { type: Date },
    union: { type: [String, Number] as PropType<string | number>, required: true },
  },
})
</script>

<template>
  <div>
    {{ optional.length }} {{ withDefault.length }} {{ obj.a }} {{ req.a.toFixed() }}
    {{ fn(1).length }} {{ fn('x') }} {{ date.getTime() }} {{ req.b }}
    <template v-if="typeof union === 'string'">{{ union.toUpperCase() }}</template>
    {{ union.toUpperCase() }}
  </div>
</template>
`,
});

write("dc-props-array-form", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: ['a', 'b'],
  data() { return { own: 1 } },
})
</script>

<template><div>{{ a.whatever }} {{ b }} {{ own.toFixed() }} {{ c }}</div></template>
`,
  "src/Parent.vue": `
<script setup lang="ts">
import Comp from './Comp.vue'
</script>

<template>
  <Comp a="1" :b="2" />
  <Comp :c="3" />
</template>
`,
});

write("dc-data-arrow-and-this", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { seed: { type: Number, required: true } },
  data: () => ({ a: 1, b: 'x' as string | null }),
  computed: {
    fromSeed(): number { return this.seed + this.a },
  },
  methods: {
    async load(): Promise<number> { return this.a },
  },
})
</script>

<template>
  <div @click="load">
    {{ a.toFixed() }} {{ b?.length }} {{ b.length }} {{ fromSeed }}
    <span v-if="b">{{ b.toUpperCase() }}</span>
  </div>
</template>
`,
});

write("dc-name-recursive-and-emits-template", {
  "src/Tree.vue": `
<script lang="ts">
import { defineComponent, type PropType } from 'vue'
interface Node { label: string; children?: Node[] }
export default defineComponent({
  name: 'Tree',
  props: { node: { type: Object as PropType<Node>, required: true } },
  emits: ['select'],
})
</script>

<template>
  <div @click="$emit('select', node)">
    {{ node.label }}
    <Tree v-for="c in node.children" :key="c.label" :node="c" @select="$emit('select', $event)" />
  </div>
</template>
`,
});
