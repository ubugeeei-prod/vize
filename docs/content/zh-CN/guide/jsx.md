---
title: JSX与多伦多证券交易所
---

<!-- Generated translation; source: guide/jsx.md -->

# JSX与多伦多证券交易所

> **状态：**JSX/TSX 涵盖编译器、打印器、类型检查器、LSP 和格式化器。
> 类型识别检查保持选择加入状态，确保 React `.tsx` 文件不会被误当为 Vue JSX。
> 独立`.jsx`/`.tsx`模块的HMR仍然是主要的集成空白。

Vize 通过**相同的编译器箱**编译 `.jsx` 和 Vue 组件 `.tsx` Vue 组件`.vue`
单文件组件——VDOM和Vapor后端，Croquis语义分析，Canon类型
检查、Patina 绒毛和 Maestro 语言服务器。没有独立的巴别管道，也没有
运行时 JSX 工厂 shim：将 JSX 组件直接降级为 Vue 渲染函数（或 Vapor
模板）由本地编译器完成。

这意味着`.tsx` Vue组件会获得相同的Rust原生编译、相同的类型检查，并且
编辑器体验和SFC一样——只是写成了类型函数而不是`<template>`。

## 启用JSX/TSX

`.jsx`和`.tsx`文件会自动通过 Vize 捆绑插件路由——没有
选择加入标记以编译它们。任何已经使用 Vize 捆绑器集成的项目都会使用 JSX/TSX
支持：

- `@vizejs/vite-plugin`
- `@vizejs/unplugin`（rollup / webpack / esbuild）
- `@vizejs/rspack-plugin`
- `@vizejs/nuxt`

```ts
// vite.config.ts — nothing JSX-specific is required
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

在底层，插件调用原生/WASM `compileJsx`入口点（从
`@vizejs/native` 和 `@vizejs/wasm`），这会降低源代码并返回渲染代码以及任意
提取了有作用域的CSS。

## 创作API

Vize JSX/TSX 组件是一个**带有类型参数的普通函数**。没有宏量营养素，也没有
`defineComponent` 封装器——类型直接从函数读取
签名并从运行时输出中擦除（零成本）。

- **Props**是**类型化的第一参数**。
- **发射和槽**是**类型化的第二参数**，Vize提供的`Ctx<Emits, Slots>`
  上下文（包含`emit`、`slots`和`attrs`，镜像Vue的设置上下文）。
- **默认道具值**来自**对参数模式的默认值进行结构化**
  编译器从结构化中提取它们。

```tsx
import { computed, ref } from "vue";

type CounterProps = {
  label: string;
  start?: number;
};

type CounterEmits = {
  change: [value: number];
};

const Counter = ({ label, start = 0 }: CounterProps, { emit }: Ctx<CounterEmits>) => {
  const count = ref(start);
  const doubled = computed(() => count.value * 2);

  const increment = () => {
    count.value += 1;
    emit("change", count.value);
  };

  return (
    <section class="counter">
      <p>
        {label}: {count.value}
      </p>
      <p>Double: {doubled.value}</p>
      <button type="button" onClick={increment}>
        Increment
      </button>
    </section>
  );
};
```

仅靠道具的组件可以完全省略第二个参数：

```tsx
const Hello = ({ name }: { name: string }) => <h1>Hello, {name}!</h1>;
```

默认值以结构化默认值的形式表示;无需单独的`props`选项：

```tsx
const Badge = ({ count = 0 }: { count?: number }) => <span class="badge">{count}</span>;
```

组件名称取自绑定（`const Counter = …`）或函数声明
（`function Card() { … }`），正如你所料。其他都是类似React的JSX元素
嵌套、片段（`<>…</>`）、表达子以及事件道具如`onClick`。唯一的
Vue特有加法是[下文](#scoped-styles)描述的`<style scoped>`元素。

> 上述仅类型创作形式是支持的通用情况。合成运行时间`props`
> 元数据和`defineComponent(() => () => vnode)`设置表单，都是计划中的后续。

## 支持JSX表面

编译器将 JSX 降级为与 SFC 模板相同的 Relief IR，然后将该 IR 发送给 VDOM
或者 Vapor 后端。这些表格均由JSX/TSX测试矩阵涵盖：

- 片段和嵌套元素
- 组件标签、成员表达标签和内在 HTML/SVG 标签
- 静态属性、动态`prop={expr}`绑定、布尔速记道具和扩展道具
- 事件处理程序，包括以道具名称编码的Vue风格选项修饰符
- `v-if`、`v-else-if`、`v-else`、`v-show`、`v-*`指令和`v-model`
- 表达式子节点、逻辑JSX分支、三元JSX分支以及`.map(...)`列表渲染
- 以对象子或渲染道具子写成的槽
- TSX 语法：类型参数、返回注释、通用 JSX 调用、cast 和非空断言
- `<style scoped>`提取;高级版支持模板字面`${expr}`插值
  但静态类和CSS变量通常更清晰

规范列表形式为惯用JSX：

```tsx
import { computed, ref } from "vue";

type Todo = {
  id: string;
  title: string;
  done: boolean;
};

type TodoListProps = {
  todos: Todo[];
  initialActiveId?: string;
};

const TodoList = ({ todos, initialActiveId }: TodoListProps) => {
  const activeId = ref(initialActiveId ?? todos[0]?.id);
  const activeTodo = computed(() => todos.find((todo) => todo.id === activeId.value));

  return (
    <section class="todo-panel">
      <header>
        <h2>{activeTodo.value?.title ?? "Select a todo"}</h2>
      </header>

      <ul class="todo-list">
        {todos.map((todo, index) => (
          <li
            key={todo.id}
            class={{ done: todo.done, active: todo.id === activeId.value }}
            data-index={index}
          >
            <button type="button" onClick={() => (activeId.value = todo.id)}>
              <span>{todo.title}</span>
              {todo.id === activeId.value ? <strong>Active</strong> : <em>{index + 1}</em>}
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
};
```

`.map(...)`回调别名（`todo`、`index`）被保留在生成类型检查器的作用域内，且
LSP 虚拟 TypeScript，因此悬停、补全、诊断和重命名都基于相同的绑定操作
你是作者。

## 输出模式：VDOM 与 Vapor

每个组件编译为**Virtual DOM**输出（Vue默认渲染器）或
[**蒸汽声**](https://blog.vuejs.org/posts/vue-vapor)输出。默认配置由配置决定;
单个组件可以覆盖它。

### 配置默认

`compiler.jsxMode`为`.jsx`/`.tsx`组件设置全局默认后端。它接受`"vdom"`
或者`"vapor"`，默认为`"vdom"`。

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    // Default every .jsx/.tsx component to Vapor output.
    jsxMode: "vapor",
  },
});
```

`jsxMode`独立于`compiler.vapor`：`vapor`切换蒸汽以控制`.vue` SFC，同时`jsxMode`
控制JSX/TSX的默认后端。项目可以将SFC保留在VDOM上，同时默认JSX为
蒸汽，或者反过来。Vite 插件也直接接受 `jsxMode` 作为插件选项，这
覆盖共享配置。

### 每个组件指令

单个组件通过指令序言覆盖默认，镜像`"use strict"`：

```tsx
// Compiled to Vapor regardless of the configured default.
const Fast = () => {
  "use vue:vapor";
  return <div class="fast" />;
};

// Compiled to Virtual DOM regardless of the configured default.
const Classic = () => {
  "use vue:vdom";
  return <div class="classic" />;
};
```

由于每个组件独立路由，**一个文件可以混合两个后端**：

```tsx
// vize.config: { compiler: { jsxMode: "vapor" } }

// No directive -> takes the configured default (Vapor here).
export const Dashboard = () => <main>{/* ... */}</main>;

// Opts back into Virtual DOM just for this component.
export const LegacyWidget = () => {
  "use vue:vdom";
  return <aside>{/* ... */}</aside>;
};
```

### 优先权

组件的输出模式按以下顺序解析：

1. 每个组件的`"use vue:vapor"`/`"use vue:vdom"`指令。
2. `compiler.jsxMode`默认设置（或插件的`jsxMode`选项）。
3. 内置的备选方案，`"vdom"`。

### 诊断

错误或冲突的指令会被报告，而不是默默无声地忽视：

- 以`"use vue:"`开头但未命名已知模式的指令（如
  `"use vue:vdomx"`）是编译错误。
- 一个组件中有两个冲突的模式指令（`"use vue:vapor"`后跟 `"use vue:vdom"`）
  被诊断;在解决模式中，第一指令仍然获胜。
- 无关序章如`"use strict"`保持不动。

## 瞄准镜样式

组件内部的`<style scoped>`元素是JSX相当于SFC的
`<style scoped>`块。它在编译时被提取——绝不会以运行时`<style>`
vnode — 其 CSS 被 scope-rewrite 了，生成了一个 `data-v-<hash>` scope id，即该 scope 属性
被注入到组件的其他元素上，重写的CSS通过
bundler 插件的 CSS 流水线。这在 VDOM 和 Vapor 后端都有效，两者都派生了
同一作用域ID，适用于某个组件。

习语上，`<style scoped>`元素在标记后排在最后——与SFC相匹配
`<template>` → `<style>`顺序——但编译器会在它出现的地方提取它。

```tsx
type CardProps = {
  title: string;
};

const Card = ({ title }: CardProps) => (
  <article class="card">
    <h2>{title}</h2>

    <style scoped>{`
      .card {
        border: 1px solid currentColor;
        padding: 12px;
      }
    `}</style>
  </article>
);
```

### 动态风格值

更倾向于使用普通类绑定、内联样式对象或CSS自定义属性来动态样式
JSX/多伦多证券交易所。支持`<style scoped>`内`${expr}`模板字面插值
类型检查，但它们是逃生出口，而非主要的创作风格：

```tsx
type BoxProps = {
  color: string;
  gap: number;
};

const Box = ({ color, gap }: BoxProps) => (
  <section
    class="box"
    style={{
      "--box-color": color,
      "--box-gap": `${gap}px`,
    }}
  >
    <p>content</p>

    <style scoped>{`
      .box {
        color: var(--box-color);
        gap: var(--box-gap);
      }
    `}</style>
  </section>
);
```

一个**没有**`scoped` 的 `<style>` 元素被视为正常元素并按原样渲染——它
没有被提取。

`<style scoped>{`.box { color： ${color}; }`}</style>`也可行，并且被类型检查器覆盖，
但保留在有作用域样式表确实需要引用组件表达式的情况下。
SFC `<style>`块内使用的字面 CSS 的 `v-bind(...)` 函数语法不被支持
在 JSX 风格模块中编写表单。

## 格式化

字形通过OXC解析器和格式化器格式化JSX/TSX脚本内容。在`.vue`档案中，
`<script lang="jsx">`、`<script lang="tsx">`和`<script setup lang="tsx">`被解析为JSX/TSX
因此，JSX 子节点和 TSX 注释格式化为
真实语法：

```vue
<script setup lang="tsx">
type CardProps = {
  title: string;
  items: string[];
};

const Card = ({ title, items }: CardProps) => (
  <section class="card">
    <h2>{title}</h2>
    {items.map((item) => (
      <span key={item}>{item}</span>
    ))}
  </section>
);
</script>
```

`vize fmt`会发现独立的`.jsx`/`.tsx`模块，并与`.vue`文件一起进行格式化
采用相同的 JSX/TSX 源类型处理：

```bash
# Formats .vue, .jsx, and .tsx files by default
vize fmt src --write
```

## 打字检查

JSX/TSX类型检查通过`typeChecker.jsxTypecheck`为**选择加入**，默认为**`false`\*\***。
默认情况下是故意关闭的：仓库可能包含 React `.tsx` 文件，而这些文件不该存在
类型校对为 Vue JSX。

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  typeChecker: {
    enabled: true,
    jsxTypecheck: true,
  },
});
```

启用后，`vize check`通过佳能对Vue组件进行类型检查`.jsx`/`.tsx`。生成的
虚拟文件是纯TypeScript，不是TSX，并且保留了作者的组件合同：

- 类型化的第一个参数仍然是props类型;
- `Ctx<Emits, Slots>`仍对设置体和JSX表达式可见;
- 事件处理程序、绑定道具、`v-if`/`v-show`、自定义指令和带作用域式插值
  表达式在使用时会以正常 TypeScript 读值的方式重新发射;
- `v-model`目标以可写的自赋值重新发射，即可读或非l值绑定
  在结合时被诊断;
- `.map(...)`列表主体在生成的回调中重新发出，因此值/索引别名得以保留
  它们推断出的元素类型。

诊断数据会在**原始源地址**报告（既有 CLI 的 JSON 格式，也有通过
LSP），因为每个有意义的虚拟 TS 区间都映射回你写的源区间。

```tsx
type FieldProps = {
  model: {
    readonly value: string;
  };
};

const Field = ({ model }: FieldProps) => <input v-model={model.value} />;
```

在上述示例中，`model.value`被检查为指派目标。如果是只读的，则
诊断任务落在TSX源代码中的`model.value`，而不是生成代码中。

```bash
# Type-check a project including its .jsx/.tsx Vue components.
# .jsx/.tsx files are collected only when typeChecker.jsxTypecheck is enabled.
vize check src
```

独立的 JSX/TSX 组件可降至纯虚拟 TypeScript 用于检查。包含以下内容的SFCs：
`<script lang="jsx">`、`<script lang="tsx">`或匹配的`script setup`块被具体化为
`.vue.tsx`虚拟文件，因此 TypeScript 解析脚本块中的 JSX 语法。LSP 和 CLI 共享
相同的降低，因此Corsa诊断在编辑器中落在相同的源区段，并且在
命令行。

## 编辑 / LSP

在有`vize lsp`支持的编辑器中打开`.jsx`/`.tsx` Vue组件，显示的语言是一样的
作为SFC的功能——**无需SFC封装**：

- 诊断
- 悬停
- 完备
- 定义的首选
- 参考文献
- 更名
- 文档符号
- 语义标记
- 代码动作
- `<style scoped>`块的嵌入式CSS诊断

结构性特征（文档符号、语义标记、带作用域的诊断、代码操作）是有效的
来自解析后的文档和始终可用。类型感知功能（诊断、悬停，
完成、进入定义、引用、重命名）仅在 `typeChecker.jsxTypecheck` 为 时才会达到
因此，React `.tsx` 文件在编辑器中也从未被视为 Vue JSX。

## 绒毛

Vize的Patina 绒毛规则在JSX/TSX上通过直接从OXC投影的零成本规则IR运行
AST\*\*。标记导向规则不会重建合成的SFC模板;它们读取JSX元素
属性直接。需要Vue模板形状的规则，比如`.map(...)`列表键检查，会运行
在倒下的救济树上方。语义规则由Croquis支持，该分析层同样用于
SFC。

这意味着JSX/TSX线处理能捕捉相同类型的问题，而无需依赖部分字符串
匹配：

```tsx
const BrokenMedia = () => (
  <article>
    <img src="/avatar.png" />
    <button accessKey="s" autoFocus>
      Save
    </button>
  </article>
);
```

上面的示例被标记为JSX源：

- `a11y/img-alt`报告失踪`alt`;
- `a11y/no-access-key`报告`accessKey`;
- `a11y/no-autofocus`报告`autoFocus`。

列出关键规则，理解JSX的惯用`.map(...)`形状：

```tsx
const KeyedList = ({ rows }: { rows: Array<{ id: string; label: string }> }) => (
  <ul>
    {rows.map((row) => (
      <li key={row.id}>{row.label}</li>
    ))}
  </ul>
);
```

诊断和修复映射到 JSX 源区间，因此 CLI 输出和编辑器装饰都指向
元素或道具应该改变。

```bash
# Lint .vue, .html, .jsx, and .tsx files
vize lint src
```

参见[静态分析](./static-analysis.md)关于lint和类型检查模型，以及
[规则](../rules/index.md)用于具体规则输出。

## 局限性

注意当前的边缘：

- **类型检查是自愿加入的。**`typeChecker.jsxTypecheck`默认`false`，因此混合使用Vue/React
  仓库不会意外将 React TSX 路由到 Vue JSX 检查器。
- **HMR 尚未为 `.jsx`/`.tsx` 模块接线。**JSX 编译器目前输出
  渲染函数模块而非完整的组件-对象模块，因此没有 Vue HMR 边界
  去依附。计划中的后续是完整的组件模块输出加上保持状态的HMR;直到
  然后，编辑到`.jsx`/`.tsx`组件时，会退回到正常的重新加载。
- **JSX `<style scoped>`块内的字面 CSS `v-bind(...)`不支持。**使用`${expr}`
  模板字面插值，即支持的类型检查形式。

## 参见

- [配置](./configuration.md) — `compiler.jsxMode`键和`typeChecker.jsxTypecheck`键，
  还有完整的共享配置图形。
- [Vite 插件](./vite-plugin.md) — 推荐的捆绑器集成。
- [静态分析](./static-analysis.md)——lint 和类型检查如何共享编译器流水线。
- [`examples/jsx-tsx`](https://github.com/ubugeeei-prod/vize/tree/main/examples/jsx-tsx) —
  专注于编译器、LINTER、类型检查器、LSP和格式化器的JSX/TSX源代码示例。
