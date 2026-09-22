---
title: Vue Rules
---

# Vue Rules

Vue rules are Patina single-file rules. They inspect SFC template structure, directive syntax,
component naming, and Vue-specific correctness hazards before the code reaches the runtime.

## `vue/require-v-for-key`

Requires every `v-for` node to have a stable key.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

Good:

```vue
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

## `vue/no-use-v-if-with-v-for`

Reports a node that has `v-if` and `v-for` at the same time. Filtering in a computed value keeps the
list identity stable and makes the template easier to analyze.

Default severity: `warning`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

Good:

```vue
<script setup lang="ts">
const visibleItems = computed(() => items.filter((item) => item.visible));
</script>

<template>
  <li v-for="item in visibleItems" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

## `vue/no-mutating-props`

Reports writes to props. The owning component should update the value through an event or a model
binding.

Default severity: `error`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

Good:

```vue
<script setup lang="ts">
const props = defineProps<{ count: number }>();
const emit = defineEmits<{ "update:count": [value: number] }>();

function increment() {
  emit("update:count", props.count + 1);
}
</script>
```

## `vue/no-v-html`

Reports `v-html` because it renders raw HTML and can turn user-controlled content into an XSS sink.

Default severity: `warning`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <article v-html="content" />
</template>
```

Good:

```vue
<template>
  <article>{{ content }}</article>
</template>
```

## `vue/no-child-content`

Reports child content on elements that also use `v-html` or `v-text`. Vue replaces the children at
runtime, so the authored content is misleading.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <p v-text="message">Fallback text</p>
</template>
```

Good:

```vue
<template>
  <p v-text="message" />
</template>
```

## `vue/no-duplicate-attributes`

Reports duplicate attributes on the same element.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <button class="primary" class="large">Save</button>
</template>
```

Good:

```vue
<template>
  <button class="primary large">Save</button>
</template>
```

## `vue/no-dupe-v-else-if`

Reports repeated conditions in a `v-if` / `v-else-if` chain.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'ready'">Still ready</p>
</template>
```

Good:

```vue
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'loading'">Loading</p>
</template>
```

## `vue/no-template-shadow`

Reports template variables that shadow variables from an outer scope. This prevents accidental
references to a different value than the reader expects.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
const item = ref("selected");
</script>

<template>
  <p v-for="item in items" :key="item.id">{{ item.name }}</p>
</template>
```

Good:

```vue
<script setup lang="ts">
const selectedItem = ref("selected");
</script>

<template>
  <p v-for="item in items" :key="item.id">{{ item.name }}</p>
</template>
```

## `vue/no-unsafe-url`

Reports URL bindings and static URL attributes that may resolve to unsafe schemes such as
`javascript:`, `vbscript:`, or executable `data:` payloads.

Default severity: `warning`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <iframe src="javascript:alert(1)"></iframe>
  <object data="data:text/html,<script>alert(1)</script>"></object>
  <img srcset="/safe.png 1x, javascript:alert(1) 2x" />
  <a :href="nextUrl">Continue</a>
</template>
```

Good:

```vue
<script setup lang="ts">
const rawNextUrl = ref("/next");
const nextUrl = computed(() => {
  return rawNextUrl.value.startsWith("/") ? rawNextUrl.value : "/";
});
</script>

<template>
  <iframe src="/embedded/report" title="Report"></iframe>
  <img srcset="/avatar.png 1x, /avatar@2x.png 2x" />
  <a :href="nextUrl">Continue</a>
</template>
```

## `vue/no-unused-components`

Reports locally registered components that never appear in the template.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <p>{{ user.name }}</p>
</template>
```

Good:

```vue
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <UserAvatar :user="user" />
</template>
```

## `vue/no-unused-properties`

Reports props declared through `defineProps` that are not used by the component.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
</template>
```

Good:

```vue
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
  <p>{{ description }}</p>
</template>
```

## `vue/require-component-is`

Reports `<component>` without an `is` binding.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <component />
</template>
```

Good:

```vue
<template>
  <component :is="currentComponent" />
</template>
```

## `vue/use-unique-element-ids`

Reports static literal IDs in places where `useId()` is safer for component reuse and SSR.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

Good:

```vue
<script setup lang="ts">
const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

## Syntax And Style Rules

Each rule below is a first-class check. The snippets are the cases the rule
actually reports, under its default options.

## `vue/attribute-hyphenation`

Enforces attribute naming style on custom components. The default, `always`,
requires hyphenated names. Native HTML attributes are not renamed.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <MyComponent myProp="value" />
  <MyComponent :myProp="value" />
</template>
```

Good:

```vue
<template>
  <MyComponent my-prop="value" />
  <MyComponent :my-prop="value" />
</template>
```

## `vue/attribute-order`

Enforces the Vue style-guide attribute order: `v-for`, then conditionals
(`v-if` / `v-else-if` / `v-else` / `v-show`), then `id`, `ref` / `key`,
`v-model`, other attributes, events, and finally `v-html` / `v-text`.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

Good:

```vue
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

## `vue/component-definition-name-casing`

Reports component filenames that are neither PascalCase nor kebab-case.
`MyComponent.vue` and `my-component.vue` are both accepted. `index.vue` and
`App.vue` are accepted too.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```text
myComponent.vue
my-Component.vue
```

Good:

```text
MyComponent.vue
my-component.vue
index.vue
App.vue
```

## `vue/component-name-in-template-casing`

Enforces component name casing in templates. The default is PascalCase.
Built-ins such as `<slot>` are left alone.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <my-component />
  <myComponent />
</template>
```

Good:

```vue
<template>
  <MyComponent />
  <RouterView />
  <slot />
</template>
```

## `vue/html-quotes`

Enforces quote style for HTML attribute values. The default, `double`,
rejects single quotes and unquoted values, including on directives.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

Good:

```vue
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

## `vue/html-self-closing`

Enforces self-closing style. By default, empty components and SVG/MathML
elements, plus HTML void elements, use a slash. Empty ordinary HTML
elements accept either closing style.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

`linter.ruleOptions["vue/html-self-closing"]` accepts `html.void`,
`html.normal`, `html.component`, `svg`, and `math`. Each value is `"always"`,
`"never"`, or `"any"`.

Bad:

```vue
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

Good:

```vue
<template>
  <MyComponent />
  <div></div>
  <div />
  <img />
  <br />
  <div>content</div>
</template>
```

## `vue/multi-word-component-names`

Requires the component filename to contain more than one word, so it cannot
collide with a present or future HTML element. `App.vue` is the exception.
Names used inside the template are not what this rule checks.

Default severity: `error`\
Presets: `essential`, `nuxt`, `opinionated`

Bad:

```text
Item.vue
Table.vue
```

Good:

```text
TodoItem.vue
DataTable.vue
App.vue
```

## `vue/mustache-interpolation-spacing`

Enforces a space inside mustache interpolation. The default, `always`,
reports each delimiter that is missing its space.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div>{{text}}</div>
  <div>{{ text}}</div>
  <div>{{text }}</div>
</template>
```

Good:

```vue
<template>
  <div>{{ text }}</div>
  <div>{{ foo.bar }}</div>
  <div>{{ foo + bar }}</div>
</template>
```

## `vue/no-boolean-attr-value`

Reports an explicit value on a boolean HTML attribute.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

Good:

```vue
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
</template>
```

## `vue/no-inline-style`

Reports inline `style` attributes. A dynamic `:style` binding is allowed,
because layout and theme values often have to be computed.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div style="color: red">Text</div>
</template>
```

Good:

```vue
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

## `vue/no-lone-template`

Reports a `<template>` element that has no structural directive. `v-if`,
`v-else-if`, `v-else`, `v-for`, and `v-slot` justify the wrapper. The SFC's
own root `<template>` block is not this element.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div>
    <template>
      <p>{{ message }}</p>
    </template>
  </div>
</template>
```

Good:

```vue
<template>
  <template v-if="ready">
    <p>{{ message }}</p>
  </template>
  <template v-for="item in items" :key="item.id">
    <p>{{ item.name }}</p>
  </template>
  <BaseCard>
    <template #header>
      <h2>{{ title }}</h2>
    </template>
  </BaseCard>
</template>
```

## `vue/no-multi-spaces`

Reports consecutive spaces in the template.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

Good:

```vue
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

## `vue/no-non-component-keep-alive-child`

Reports a plain element directly below `<KeepAlive>`. Vue caches component
vnodes, so a native wrapper leaves the component uncached. A wrapper whose
only directive is `v-show` is ignored. `v-show` together with something that
changes identity, such as `:key`, is still reported.

Default severity: `warning`\
Presets: none (opt-in)

Bad:

```vue
<template>
  <KeepAlive>
    <div v-if="ready">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

Good:

```vue
<template>
  <KeepAlive>
    <UserCard v-if="ready" />
  </KeepAlive>
  <KeepAlive>
    <div v-show="opened">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

## `vue/no-preprocessor-lang`

Reports `lang="sass"`, `lang="scss"`, `lang="less"`, `lang="stylus"`, and
`lang="styl"` on a `<style>` block.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<style lang="scss">
.button {
  color: red;
}
</style>
```

Good:

```vue
<style scoped>
.button {
  color: red;
}
</style>
```

## `vue/no-reserved-component-names`

Reports an explicit component name that is an HTML element, an SVG element,
or a Vue built-in. It reads the Options API `name` and
`defineOptions({ name })`. It does not read the filename, and using
`<Transition>` or `<KeepAlive>` in a template is fine.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<script>
export default {
  name: "button",
};
</script>
```

```vue
<script setup lang="ts">
defineOptions({ name: "svg" });
</script>
```

Good:

```vue
<script setup lang="ts">
defineOptions({ name: "AppButton" });
</script>

<template>
  <Transition>
    <AppButton />
  </Transition>
</template>
```

## `vue/no-script-non-standard-lang`

Reports a `<script>` language other than `js`, `jsx`, `ts`, `tsx`, or
`typescript`, matched without regard to case. Omitting `lang` is JavaScript
and is allowed.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<script lang="coffee">
# CoffeeScript
</script>
```

Good:

```vue
<script setup lang="ts">
const label = "Save";
</script>
```

## `vue/no-src-attribute`

Reports `src` on the SFC's `<template>`, `<script>`, or `<style>` block.
The component source should live in the `.vue` file.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template src="./template.html"></template>
<script src="./script.ts"></script>
<style src="./style.css"></style>
```

Good:

```vue
<template>
  <p>Hello</p>
</template>

<script setup lang="ts">
const label = "Hello";
</script>

<style scoped>
p {
  color: red;
}
</style>
```

## `vue/no-template-key`

Reports `key` on a `<template>` element that is not a `v-for`. A `v-for`
template may carry the key. Put `key` on a real element otherwise.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <template :key="section">
    <div>{{ item }}</div>
  </template>
</template>
```

Good:

```vue
<template>
  <template v-for="item in items" :key="item.id">
    <div>{{ item.name }}</div>
  </template>
  <section :key="section">
    <div>{{ item }}</div>
  </section>
</template>
```

## `vue/no-template-lang`

Reports `lang="pug"`, `lang="jade"`, `lang="slm"`, or `lang="haml"` on the
`<template>` block.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template lang="pug">
div.container
  h1 Hello
</template>
```

Good:

```vue
<template>
  <div class="container">
    <h1>Hello</h1>
  </div>
</template>
```

## `vue/no-textarea-mustache`

Reports mustache interpolation inside `<textarea>`. The text does not bind
the control. Use `v-model`.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <textarea>{{ message }}</textarea>
</template>
```

Good:

```vue
<template>
  <textarea v-model="message"></textarea>
</template>
```

## `vue/no-unused-vars`

Reports a variable introduced by `v-for` or `v-slot` that the template never
reads. A name that starts with `_` is treated as intentionally unused.

Default severity: `warning`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <li v-for="(item, index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ foo }">
    <span>Hello</span>
  </template>
</template>
```

Good:

```vue
<template>
  <li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
  <li v-for="(item, _index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ data }">
    <span>{{ data }}</span>
  </template>
</template>
```

## `vue/no-useless-template-attributes`

Reports an attribute on `<template>` that Vue ignores. A `<template>` element
only keeps structural directives (`v-if`, `v-else-if`, `v-else`, `v-for`,
`v-slot`) and the `key` that belongs to a `v-for`.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <BaseCard>
    <template v-if="show" class="panel"><div /></template>
    <template v-for="item in items" id="list"><div /></template>
    <template #header ref="header"><div /></template>
  </BaseCard>
</template>
```

Good:

```vue
<template>
  <BaseCard>
    <template v-if="show"><div /></template>
    <template v-for="item in items" :key="item.id"><div /></template>
    <template #header><div /></template>
  </BaseCard>
</template>
```

## `vue/no-v-text-v-html-on-component`

Reports `v-text` or `v-html` on a component. Those directives replace the
component's own output. A native element, or `<component is="div">`, may use
them.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

Good:

```vue
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
</template>
```

## `vue/permitted-contents`

Reports HTML nesting the parser or the content model forbids, so the DOM and
the virtual DOM would disagree. An unresolved component, a slot, or a dynamic
binding stays silent rather than guessing.

Default severity: `error`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <p><div>block in a paragraph</div></p>
  <table><tr><td>row without tbody</td></tr></table>
  <a href="#"><button type="button">nested control</button></a>
  <ul><div>not a list item</div></ul>
</template>
```

Good:

```vue
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
</template>
```

## `vue/prefer-props-shorthand`

Reports a prop binding whose name matches the value expression. Vue 3.4
writes that as `:foo` instead of `:foo="foo"`. A kebab-case attribute matches
its camelCase value: `:user-name="userName"` is the same case.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <MyComponent :foo="foo" />
  <MyComponent :user-name="userName" />
</template>
```

Good:

```vue
<template>
  <MyComponent :foo />
  <MyComponent :user-name />
  <MyComponent :foo="bar" />
</template>
```

## `vue/prop-name-casing`

Enforces camelCase for prop names in this component's own `defineProps`
declaration. How a parent writes the attribute is `vue/attribute-hyphenation`,
not this rule.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
defineProps({ "my-prop": String });
</script>
```

Good:

```vue
<script setup lang="ts">
defineProps({ myProp: String });
</script>
```

## `vue/require-component-registration`

Reports a component used in the template that was not imported in
`<script setup>` and was not registered. Built-ins such as `<component>` and
`<Transition>` are ignored.

Default severity: `warning`\
Presets: `opinionated`

Bad:

```vue
<script setup lang="ts">
// MyButton is never imported.
</script>

<template>
  <MyButton>Save</MyButton>
</template>
```

Good:

```vue
<script setup lang="ts">
import MyButton from "./MyButton.vue";
</script>

<template>
  <MyButton>Save</MyButton>
</template>
```

## `vue/require-scoped-style`

Reports a `<style>` block that is neither `scoped` nor `module`.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<style>
.button {
  color: red;
}
</style>
```

Good:

```vue
<style scoped>
.button {
  color: red;
}
</style>
```

```vue
<style module>
.button {
  color: red;
}
</style>
```

## `vue/scoped-event-names`

Recommends `context:event` for component listeners whose names end in a
known context such as `Audio`, `Form`, or `Dialog`. A single `@playAudio`
listener is enough to trigger the rule; native element events are skipped.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <AudioPlayer
    @playAudio="play"
    @pauseAudio="pause"
    @reloadAudio="reload"
  />
</template>
```

Good:

```vue
<template>
  <AudioPlayer
    @audio:play="play"
    @audio:pause="pause"
    @audio:reload="reload"
  />
</template>
```

## `vue/sfc-element-order`

Enforces `vue/block-order`'s default: `<script>` and `<template>` may come in
either order, and `<style>` is last.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<style scoped>
.panel {
  color: red;
}
</style>
<script setup lang="ts">
const label = "Save";
</script>
```

Good:

```vue
<script setup lang="ts">
const label = "Save";
</script>

<template>
  <p>{{ label }}</p>
</template>

<style scoped>
p {
  color: red;
}
</style>
```

```vue
<template>
  <p>{{ label }}</p>
</template>

<script setup lang="ts">
const label = "Save";
</script>

<style scoped></style>
```

## `vue/single-style-block`

Reports more than one `<style>` block when those blocks have the same
purpose. One `scoped` block and one unscoped block are allowed.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<style scoped>
.panel {
  color: red;
}
</style>

<style scoped>
.title {
  color: blue;
}
</style>
```

Good:

```vue
<style scoped>
.panel {
  color: red;
}
.title {
  color: blue;
}
</style>
```

## `vue/use-v-on-exact`

Reports an unmodified listener that sits beside the same event with a key
modifier. Without `.exact`, the plain listener also runs when the modifier
matches.

Default severity: `warning`\
Presets: `essential`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

Good:

```vue
<template>
  <button
    type="button"
    @click.exact="handleClick"
    @click.ctrl="handleCtrlClick"
  >
    Save
  </button>
</template>
```

## `vue/v-bind-style`

Enforces `v-bind` style. The default, `shorthand`, wants `:attr` rather than
`v-bind:attr`. `longform` is the opposite.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

Good:

```vue
<template>
  <div :class="panelClass"></div>
</template>
```

## `vue/v-on-style`

Enforces `v-on` style. The default, `shorthand`, wants `@event` rather than
`v-on:event`.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-on:click="handleClick"></div>
</template>
```

Good:

```vue
<template>
  <div @click="handleClick"></div>
</template>
```

## `vue/v-slot-style`

Enforces `v-slot` style per position. The default wants `v-slot` on the
component for the default slot, and `#name` on a `<template>` for a named
slot. `#default` on the component, and `v-slot:name` on a template, are the
reported shapes.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <MyComponent #default="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template v-slot:header>Header</template>
  </MyComponent>
</template>
```

Good:

```vue
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

## `vue/valid-attribute-name`

Reports invalid characters in attribute names, such as quotes, whitespace,
or control characters. A leading digit is allowed.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div my"attr="value"></div>
</template>
```

Good:

```vue
<template>
  <div my-attr="value"></div>
  <div data-value="value"></div>
</template>
```

## `vue/valid-v-bind`

Reports `v-bind` with no argument and no object expression, and an empty `:`
shorthand. A Vue 3.4 same-name shorthand (`:loading`) is valid.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

Good:

```vue
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

## `vue/valid-v-else`

Reports `v-else` with an expression, `v-else` combined with `v-if` on the
same element, and `v-else` that does not follow `v-if` or `v-else-if`.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

Good:

```vue
<template>
  <div v-if="ready"></div>
  <div v-else></div>
</template>
```

## `vue/valid-v-for`

Reports `v-for` with no expression, an empty expression, or a modifier.
The expression has to use `in` or `of`.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-for></div>
  <div v-for=""></div>
  <div v-for.stop="item in items"></div>
</template>
```

Good:

```vue
<template>
  <div v-for="item in items" :key="item.id"></div>
  <div v-for="(item, index) of items" :key="index"></div>
</template>
```

## `vue/valid-v-if`

Reports `v-if` with no expression, an empty expression, or `v-if` combined
with `v-else` / `v-else-if` on the same element.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-if></div>
  <div v-if=""></div>
  <div v-if="ready" v-else></div>
</template>
```

Good:

```vue
<template>
  <div v-if="ready"></div>
  <div v-if="count > 0"></div>
</template>
```

## `vue/valid-v-memo`

Reports `v-memo` without a dependency-array expression.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-memo></div>
</template>
```

Good:

```vue
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

## `vue/valid-v-model`

Reports `v-model` without an expression, and `v-model` on an element that
cannot host it. `input`, `select`, `textarea`, and components can.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

Good:

```vue
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
</template>
```

## `vue/valid-v-on`

Reports `v-on` with no event name and no handler. Object syntax
(`v-on="{ click: onClick }"`) is a handler without one event name, and it is
valid.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
</template>
```

Good:

```vue
<template>
  <div @click="handleClick"></div>
  <div v-on="{ click: handleClick }"></div>
</template>
```

## `vue/valid-v-show`

Reports `v-show` without an expression, and `v-show` on `<template>`.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-show></div>
  <template v-show="ready"><div></div></template>
</template>
```

Good:

```vue
<template>
  <div v-show="ready"></div>
  <div v-show="count > 0"></div>
</template>
```

## `vue/valid-v-slot`

Reports `v-slot` on a native element, two slot directives on the same
component, and two named slots on one `<template>`.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div v-slot:header></div>
  <MyComponent v-slot v-slot:header />
  <template v-slot:header v-slot:footer />
</template>
```

Good:

```vue
<template>
  <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

## `vue/warn-custom-block`

Reports a top-level SFC block other than `<script>`, `<template>`, and
`<style>`. An element inside `<template>` is not a custom block. The finding
is a warning that the block needs a plugin, not a parse error.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<i18n>
{ "en": { "hello": "Hello" } }
</i18n>

<template>
  <p>{{ hello }}</p>
</template>
```

Good:

```vue
<template>
  <p>{{ hello }}</p>
</template>

<script setup lang="ts">
const hello = "Hello";
</script>
```

## `vue/warn-custom-directive`

Reports a directive that is not a Vue built-in. The directive still has to
be registered for the app to run. Built-ins such as `v-if`, `v-model`, and
`@click` are not reported.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <input v-focus />
  <input v-mask="'###-####'" />
  <div v-click-outside="handleClose"></div>
</template>
```

Good:

```vue
<template>
  <div v-if="ready"></div>
  <input v-model="value" />
  <button type="button" @click="onClick">Save</button>
</template>
```
