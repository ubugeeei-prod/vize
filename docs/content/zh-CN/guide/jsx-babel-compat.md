---
title: Babel JSX 兼容性
---

<!-- Generated translation; source: guide/jsx-babel-compat.md -->

# Babel JSX 兼容性

> **状态：**选择加入，默认关闭。配置加载器、native/WASM `compileJsx` 绑定和 Vize bundler 插件
> 都支持 `compiler.jsxCompat`。

Vize 通过自己的编译器箱编译 `.jsx` 和 `.tsx` ，因此输出呈现
模板编译器形状：块树，从 JavaScript 中降 `v-if` / `v-for` ，并在每个节点上补丁
标志。 [`@vue/babel-plugin-jsx`](https://github.com/vuejs/babel-plugin-jsx) 完全不做这些
——它发出裸 `createVNode` 调用，从不打开任何块，保持 `&&`、 `?:` 和 `.map()` 为
普通JavaScript，默认情况下完全不发出补丁标志。

大部分差异在运行时是看不见的。剩下的就是这个交换机存在的意义：
迁移出 Babel 插件的项目需要一种方式来请求插件的语义，而不是 Vize 的。
`compiler.jsxCompat: "babel"`是那个开关。

本页讨论 **兼容性语义**。关于创作API、类型表面和
Vapor/VDOM输出选择器，请参见 [JSX & TSX guide](./jsx.md)。

## 启用它

```json
{
  "compiler": {
    "jsxCompat": "babel"
  }
}
```

密钥接受`"native"`（默认值）和`"babel"`。其他值会退回到`"native"`
，而不是构建失败，这与未识别`jsxMode`的处理方式相符：一个零散的配置
值绝不能阻碍编译。

相同的值也可以直接传给 `compileJsx` 绑定：

```js
import { compileJsx } from "@vizejs/native";

const result = compileJsx(source, {
  filename: "App.tsx",
  lang: "tsx",
  jsxCompat: "babel",
});
```

`@vizejs/wasm` 也暴露同样的 `jsxCompat` 选项。Vite、unplugin、Rspack 和 Nuxt 入口会把各自
配置的 `jsxCompat` 传给 `compileJsx`，其选项类型也允许把 `jsxCompat` 与 `jsxMode`、`vapor`
并列直接指定。

## 为什么它是选择加入和项目级别的

**默认关闭。**`"native"`是默认，必须保持默认状态。翻转它会
无声地改变所有现有 Vize 项目的输出，而这些项目都不需要 babel
语义。

**项目级，没有每个组件的表格。**`jsxMode`可以按组件选择，并附带
`"use vue:vapor"`/`"use vue:vdom"`序章，因为 VDOM 和 Vapor 组件在一个模块中可以愉快共存
——每个模块都是独立的渲染函数。兼容模式不是那样的。它
改变**模块级**的输出形状：Babel 插件会原地重写 JSX 表达式，使
`const A = () => <div />` 保持为 `const A = …`，而 Vize 则输出独立的 `render` 导出。一个
模块一半编译为兼容模式，另一半脱离，会从一个文件中输出两个互不兼容的模
形状。因此，Compat只为该项目配置一次，并且有意不设置
指令序章。

## 插件选项映射

Babel插件本身的选项在Vize中没有配置文件拼写。每个都是
[`vize_atelier_jsx`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_jsx)箱
`compile_jsx_with_babel_*`入口点的参数，
，除非`jsxCompat``"babel"`。

| `@vue/babel-plugin-jsx` | Vize入口                                    |
| ----------------------- | ------------------------------------------- |
| `transformOn`           | `BabelJsxOptions::transform_on`             |
| `pragma`                | `compile_jsx_with_babel_pragma`             |
| `mergeProps`            | `compile_jsx_with_babel_merge_props`        |
| `isCustomElement`       | `BabelJsxCustomizations::is_custom_element` |
| `enableObjectSlots`     | `compile_jsx_with_babel_object_slots`       |
| 任意组合                | `compile_jsx_with_babel_customizations`     |

表格中没有两个插件选项：

- **`optimize`**没有 Vize 的对应产品，因为 Vize 的输出总是经过优化——这正是
  插件的 `optimize: true` 产生了什么。插件默认是 `optimize: false`，其
  说明警告开启后“可能会跳过某些重新渲染”，因此间隙兼容模式必须
  关闭，这才是 _未优化_ 的方向：输出无补丁标志的输出。
- **`resolveType`**未被实现;详见下文“推迟的事项”。

`enableObjectSlots`默认在插件和 Vize 的兼容通道中 `true`：作为组件唯一子节点传递的单个标识符或
调用表达式可能已经是 slots 对象，因此运行时会
检查。传递`false`总是将该值视为原始默认槽子。

## 当该模式不适用时

**Vapor 输出。**`@vue/babel-plugin-jsx` 是 vdom 时代的一个插件：它定义的每个输出形状都是一个
`createVNode` 树，且没有 Vapor 的对应物。因此，`jsxCompat: "babel"`与
`jsxMode: "vapor"`结合没有明确的含义，并且通过诊断性而非
默默忽视来拒绝：

```text
compiler.jsxCompat: "babel" is not supported with Vapor output: @vue/babel-plugin-jsx has no
Vapor equivalent. Use jsxMode "vdom" for babel compatibility, or drop jsxCompat to use Vize's own
Vapor semantics.
```

**SSR输出。**插件的选项描述了客户端的 vnode 树。因此，SSR编译
完全不应用 Babel 通道——不应用`transformOn`和`enableObjectSlots`辅助、
`isCustomElement`谓词、`mergeProps: false`以及所有仅 Babel 的降级——并使用Vize
自身的SSR语义，而不是输出半应用的混合。

这两点都是刻意的回答，记录在箱子里，避免被重新争论。

## 推迟的事项

两行语料库列被记录为 `deferred` 而非发散，因为它们都在等待
无关的编译器工作，而非兼容模式：

| 行                        | 巴别塔的功效                          | 它正在等待什么                                                                                                                                                         |
| ------------------------- | ------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `options/resolve_type_on` | 附录 `{ props: { … }, name: "A" }`    | 类型驱动的道具/发射推理，需要在[#1497](https://github.com/ubugeeei-prod/vize/issues/1497) / [#1502](https://github.com/ubugeeei-prod/vize/issues/1502)上跟踪类型分辨率 |
| `slots/dynamic_slot_name` | 发射计算出的密钥， `{ [n]: () => … }` | 动态插槽降级；Vize 目前会警告并丢弃该插槽                                                                                                                              |

## 兼容性的衡量方式

兼容性是以 **真实插件**为标准，而不是凭记忆。语料库由
钉置的 `@vue/babel-plugin-jsx`编译，其输出被记录为坚定的真实数据，Rust套件
将该记录快照与Vize的输出并列，每行明确判决。

| 文物                                                              | 职责                                    |
| ----------------------------------------------------------------- | --------------------------------------- |
| `crates/vize_atelier_jsx/tests/babel_compat/fixtures/corpus.json` | 输入和插件选项都被编译为                |
| `crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs`           | 通过真实插件运行语料库                  |
| `crates/vize_atelier_jsx/tests/babel_compat_oracle.rs`            | 每行快照 Babel 的输出与 Vize 的输出并列 |
| `crates/vize_atelier_jsx/tests/BABEL_COMPAT_INVENTORY.md`         | 判决表的散文形式，以及总数              |

逐行判决、几乎每行都成立的全局分歧（模块形状、块
树、补丁标志、未降低控制流）以及当前总数都存在于
[`BABEL_COMPAT_INVENTORY.md`](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_atelier_jsx/tests/BABEL_COMPAT_INVENTORY.md)中。
这些总数是通过`babel_compat_verdict_totals`测试固定的，因此不会偏离
语料库——这也是本页没有引用任何一个的原因。请直接阅读原文。

要在本地重新生成或验证录音：

```bash
node crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs --check
cargo test -p vize_atelier_jsx --test babel_compat_oracle
node --test tests/tooling/babel-jsx-oracle.test.ts
```

## 参见

- [JSX & TSX](./jsx.md) — 创作API、类型道具和发射器、作用域样式以及 `jsxMode`。
- [Configuration](./configuration.md) ——每个 `compiler.*` 键和配置文件查找顺序。
- [`examples/jsx-tsx`](https://github.com/ubugeeei-prod/vize/tree/main/examples/jsx-tsx) ——一个可运行的JSX/TSX项目。
