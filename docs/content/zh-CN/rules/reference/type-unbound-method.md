---
title: "type/unbound-method"
description: "禁止在没有接收者（`this`）的情况下引用方法。"
---

# `type/unbound-method`

禁止在没有接收者（`this`）的情况下引用方法。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

counter.increment 取下了方法而没有带上接收者，之后调用时 this 会丢失。

```vue annotate="remove:9"
<script setup lang="ts">
class Counter {
  value = 0;
  increment(): void {
    this.value += 1;
  }
}
const counter = new Counter();
const unbound = counter.increment;
</script>
```

## 正确示例

counter.increment() 在 counter 上调用方法，因此 this 仍然绑定。

```vue annotate="add:9"
<script setup lang="ts">
class Counter {
  value = 0;
  increment(): void {
    this.value += 1;
  }
}
const counter = new Counter();
counter.increment();
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/unbound-method": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 与 typescript-eslint 的差异

不报告从全局对象读取的库方法。这适用于类型为 `Window & typeof globalThis` 或 `typeof globalThis` 的接收者，包括 `window.setTimeout`、`const { matchMedia } = window`、`self.requestAnimationFrame` 和 `document.defaultView?.setTimeout`。平台操作在没有接收者时会使用全局对象，因此这些引用可以照常运行。你自己的代码添加到 `Window` 上的方法仍会被报告。另一个框架的 `Window`（`parent.setTimeout`）以及通过全局对象取得的对象的方法（`window.localStorage.getItem`）同样会被报告。设置 `checkGlobalObjectMethods: true` 后，这些引用会像在 typescript-eslint 中一样被报告。

被合并了声明的内置对象（`declare global { interface Math {} }`）仍然享有豁免；通过任何指向 `RegExp` 构造函数的路径访问 `RegExp.escape` 都会被豁免。豁免依据的是 TypeScript 库的声明，而不是运行 linter 的 Node.js 版本。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/unbound_method.rs#L5)
