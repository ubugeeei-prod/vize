// Options API matrix, part 2: checkJs, mixed setup, functional/generic, class components.
import { write } from "./cases1.ts";

const JS_TSCONFIG = JSON.stringify({
  compilerOptions: {
    target: "ESNext", module: "ESNext", moduleResolution: "bundler", strict: true, noEmit: true,
    skipLibCheck: true, jsx: "preserve", lib: ["ESNext", "DOM"], types: [], allowJs: true, checkJs: true,
  },
  include: ["src/**/*.ts", "src/**/*.vue", "src/**/*.js"],
  contentMappers: [{ package: "vize", extensions: [".vue"] }],
}, null, 2);

write("js-plain-checkjs", {
  "tsconfig.json": JS_TSCONFIG,
  "src/Comp.vue": `
<script>
export default {
  props: {
    msg: { type: String, required: true },
    count: { type: Number, default: 0 },
  },
  emits: ['done'],
  data() {
    return { local: 1, text: 'a' }
  },
  computed: {
    upper() { return this.msg.toUpperCase() },
  },
  methods: {
    bump(n) { this.local += n },
  },
}
</script>

<template>
  <div>
    {{ msg.toUpperCase() }} {{ count.toFixed() }} {{ local }} {{ upper.length }}
    <button @click="bump(1)">+</button>
    {{ msg.toFixed() }} {{ text.toFixed() }} {{ nope }}
  </div>
</template>
`,
});

write("js-dc-checkjs", {
  "tsconfig.json": JS_TSCONFIG,
  "src/Comp.vue": `
<script>
import { defineComponent } from 'vue'
export default defineComponent({
  props: {
    msg: { type: String, required: true },
    count: { type: Number, default: 0 },
  },
  data() {
    return { local: 1 }
  },
  computed: {
    /** @returns {string} */
    upper() { return this.msg.toUpperCase() },
  },
  methods: {
    /** @param {number} n */
    bump(n) { this.local += n },
  },
})
</script>

<template>
  <div>
    {{ msg.toUpperCase() }} {{ count.toFixed() }} {{ local }} {{ upper.length }}
    <button @click="bump(1)">+</button>
    <button @click="bump('x')">x</button>
    {{ msg.toFixed() }} {{ nope }}
  </div>
</template>
`,
  "src/Parent.vue": `
<script>
import { defineComponent } from 'vue'
import Comp from './Comp.vue'
export default defineComponent({ components: { Comp } })
</script>

<template>
  <Comp msg="a" />
  <Comp :msg="1" />
  <Comp />
</template>
`,
});

write("js-plain-no-checkjs", {
  "tsconfig.json": JSON.stringify({
    compilerOptions: { target: "ESNext", module: "ESNext", moduleResolution: "bundler", strict: true,
      noEmit: true, skipLibCheck: true, lib: ["ESNext", "DOM"], types: [], allowJs: true },
    include: ["src/**/*.ts", "src/**/*.vue"],
    contentMappers: [{ package: "vize", extensions: [".vue"] }],
  }, null, 2),
  "src/Comp.vue": `
<script>
export default {
  props: { msg: String },
  data() { return { n: 1 } },
}
</script>

<template><div>{{ msg.toFixed() }} {{ n.toUpperCase() }} {{ nope }}</div></template>
`,
  "src/Use.vue": `
<script setup lang="ts">
import Comp from './Comp.vue'
</script>

<template><Comp :msg="1" /></template>
`,
});

write("mixed-options-and-setup", {
  "src/Comp.vue": `
<script lang="ts">
export default {
  name: 'Mixed',
  inheritAttrs: false,
}
</script>

<script setup lang="ts">
const props = defineProps<{ label: string; count?: number }>()
const emit = defineEmits<{ pick: [n: number] }>()
</script>

<template>
  <div v-bind="$attrs" @click="emit('pick', 1)">
    {{ label.toUpperCase() }} {{ props.count?.toFixed() }} {{ $attrs.id }}
    {{ label.toFixed() }} {{ count.toFixed() }}
  </div>
</template>
`,
  "src/Use.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import Comp from './Comp.vue'
export default defineComponent({
  components: { Comp },
  methods: { onPick(n: number) { return n } },
})
</script>

<template>
  <Comp label="x" @pick="onPick" />
  <Comp :label="1" />
</template>
`,
});

write("mixed-dc-normal-script-and-setup", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import Icon from './Icon.vue'
export const shared = 1
export default defineComponent({
  components: { Icon },
})
</script>

<script setup lang="ts">
defineProps<{ name: string }>()
</script>

<template>
  <Icon :name="name" /> {{ shared }}
  <Icon :name="1" />
</template>
`,
  "src/Icon.vue": `
<script setup lang="ts">
defineProps<{ name: string }>()
</script>

<template><i>{{ name }}</i></template>
`,
});

write("functional-component", {
  "src/Fn.ts": `
import { h, type FunctionalComponent } from 'vue'
export const Fn: FunctionalComponent<{ msg: string }, { pick: (n: number) => void }> = (props, { emit }) =>
  h('div', { onClick: () => emit('pick', 1) }, props.msg)
Fn.props = ['msg']
Fn.emits = ['pick']
`,
  "src/Use.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import { Fn } from './Fn'
export default defineComponent({
  components: { Fn },
  methods: { onPick(n: number) { return n } },
})
</script>

<template>
  <Fn msg="a" @pick="onPick" />
  <Fn :msg="1" />
</template>
`,
});

write("generic-define-component-function", {
  "src/Gen.ts": `
import { defineComponent, h } from 'vue'
export const Gen = defineComponent(
  <T extends string | number>(props: { item: T; list: T[] }) => () => h('div', String(props.item)),
  { props: ['item', 'list'] },
)
`,
  "src/Use.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import { Gen } from './Gen'
export default defineComponent({ components: { Gen } })
</script>

<template>
  <Gen :item="1" :list="[1, 2]" />
  <Gen item="a" :list="['b']" />
  <Gen :item="1" :list="['b']" />
</template>
`,
});

write("class-vue-facing-decorator", {
  "src/Comp.vue": `
<script lang="ts">
import { Component, Prop, Vue, Emit, Watch } from 'vue-facing-decorator'
@Component
export default class Comp extends Vue {
  @Prop({ required: true }) readonly msg!: string
  @Prop({ default: 0 }) readonly size!: number
  count = 0
  get double(): number { return this.count * 2 }
  inc(by: number) { this.count += by }
  @Emit('pick') pick(n: number) { return n }
  @Watch('count') onCount(n: number) { return n }
}
</script>

<template>
  <div>
    {{ msg.toUpperCase() }} {{ size.toFixed() }} {{ count.toFixed() }} {{ double.toFixed() }}
    <button @click="inc(1)">+</button>
    <button @click="pick(2)">pick</button>
  </div>
</template>
`,
  "src/CompErrors.vue": `
<script lang="ts">
import { Component, Prop, Vue } from 'vue-facing-decorator'
@Component
export default class CompErrors extends Vue {
  @Prop({ required: true }) readonly msg!: string
  count = 0
  inc(by: number) { this.count += by }
}
</script>

<template>
  <div>
    {{ msg.toFixed() }} {{ count.toUpperCase() }} {{ nope }}
    <button @click="inc('x')">+</button>
  </div>
</template>
`,
});

write("class-vue-property-decorator", {
  "src/Comp.vue": `
<script lang="ts">
import { Options, Vue } from 'vue-class-component'
import { Prop, Emit, Watch, Model, PropSync } from 'vue-property-decorator'
@Options({})
export default class Comp extends Vue {
  @Prop({ required: true }) readonly msg!: string
  @Prop({ type: Number, default: 0 }) readonly size!: number
  @Model('modelValue', { type: String }) readonly value!: string
  @PropSync('open', { type: Boolean }) syncedOpen!: boolean
  count = 0
  get double(): number { return this.count * 2 }
  inc(by: number) { this.count += by }
  @Emit('pick') pick(n: number) { return n }
  @Watch('count') onCount(n: number) { return n }
}
</script>

<template>
  <div>
    {{ msg.toUpperCase() }} {{ size.toFixed() }} {{ count.toFixed() }} {{ double.toFixed() }}
    {{ value }} {{ syncedOpen }}
    <button @click="inc(1)">+</button>
    <button @click="pick(2)">pick</button>
    <button @click="syncedOpen = !syncedOpen">t</button>
  </div>
</template>
`,
  "src/CompErrors.vue": `
<script lang="ts">
import { Options, Vue } from 'vue-class-component'
import { Prop } from 'vue-property-decorator'
@Options({})
export default class CompErrors extends Vue {
  @Prop({ required: true }) readonly msg!: string
  count = 0
  inc(by: number) { this.count += by }
}
</script>

<template>
  <div>
    {{ msg.toFixed() }} {{ count.toUpperCase() }} {{ nope }}
    <button @click="inc('x')">+</button>
  </div>
</template>
`,
});

write("plain-object-generic-annotation", {
  "src/Comp.vue": `
<script lang="ts">
import type { ComponentOptions } from 'vue'
const options: ComponentOptions = {
  data() { return { n: 1 } },
}
export default options
</script>

<template><div>{{ n }} {{ anything }}</div></template>
`,
});

write("dc-with-satisfies-and-const", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
const Comp = defineComponent({
  props: { a: { type: String, required: true } },
  data() { return { b: 1 } },
})
export default Comp
</script>

<template><div>{{ a.toUpperCase() }} {{ b.toFixed() }} {{ a.toFixed() }} {{ c }}</div></template>
`,
});

write("plain-ts-template-errors-in-script-member", {
  "src/Comp.vue": `
<script lang="ts">
export default {
  props: { a: { type: String, required: true } },
  data() { return { b: 1 } },
  computed: { c(): string { return this.a + this.b } },
  methods: { m(x: number) { return x + this.b } },
}
</script>

<template>
  <div>{{ a.toUpperCase() }} {{ c.length }} {{ m(1) }} {{ m('x') }} {{ b.length }}</div>
</template>
`,
});

write("dc-emits-with-v-on-listeners-in-parent", {
  "src/Child.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  emits: ['changed', 'update:value'],
  props: { value: String },
})
</script>

<template><div /></template>
`,
  "src/Parent.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import Child from './Child.vue'
export default defineComponent({
  components: { Child },
  data() { return { v: 'x' } },
  methods: { onChanged() {} },
})
</script>

<template>
  <Child @changed="onChanged" v-model:value="v" @unknown-event="onChanged" />
  <Child @changed="missingHandler" />
</template>
`,
});

write("dc-props-type-only-constructor", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
class Point { constructor(public x: number, public y: number) {} }
export default defineComponent({
  props: {
    s: String, n: Number, b: Boolean, a: Array, o: Object, f: Function, sym: Symbol, p: Point,
  },
})
</script>

<template>
  <div>{{ s?.length }} {{ n?.toFixed() }} {{ b }} {{ a?.length }} {{ o }} {{ p?.x }} {{ p?.z }}</div>
</template>
`,
});

write("dc-template-v-for-on-computed-and-methods", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  data() { return { rows: [{ id: 1, name: 'a' }] } },
  computed: { sorted(): { id: number; name: string }[] { return [...this.rows] } },
  methods: { fmt(r: { id: number }) { return r.id.toFixed() } },
})
</script>

<template>
  <ul>
    <li v-for="r in sorted" :key="r.id">{{ fmt(r) }} {{ r.name.toUpperCase() }} {{ r.missing }}</li>
    <li v-for="(r, i) in rows" :key="i">{{ i.toFixed() }} {{ fmt({ id: 'x' }) }}</li>
  </ul>
</template>
`,
});

write("dc-options-only-render-no-template", {
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent, h } from 'vue'
export default defineComponent({
  props: { a: { type: String, required: true } },
  render() { return h('div', this.a.toFixed()) },
})
</script>
`,
});
