---
title: "petite-vue 规则"
---

# petite-vue 规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。


| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](#petite-vue-no-unsupported-directive) | [错误示例](#petite-vue-no-unsupported-directive-bad) · [正确示例](#petite-vue-no-unsupported-directive-good) | 禁止 petite-vue 不支持的指令 |
| [`petite-vue/valid-v-effect`](#petite-vue-valid-v-effect) | [错误示例](#petite-vue-valid-v-effect-bad) · [正确示例](#petite-vue-valid-v-effect-good) | 要求 v-effect 具有非空表达式 |
| [`petite-vue/valid-v-scope`](#petite-vue-valid-v-scope) | [错误示例](#petite-vue-valid-v-scope-bad) · [正确示例](#petite-vue-valid-v-scope-good) | 要求 v-scope 绑定对象字面量 |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `petite-vue/no-unsupported-directive`

禁止 petite-vue 不支持的指令

[错误示例](#petite-vue-no-unsupported-directive-bad) · [正确示例](#petite-vue-no-unsupported-directive-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: 检测为 petite-vue 的 HTML 文档；普通 Vue SFC 不在此规则范围内。  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/no-unsupported-directive": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-no-unsupported-directive-bad"></span>

**错误示例**

`v-memo`、`v-slot:header` 和自定义 `v-my-directive` 不在 petite-vue 支持的指令列表中。petite-vue 脚本将此 HTML 标识为相应方言。

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-memo="[a, b]"></div>
<template v-slot:header></template>
<div v-my-directive></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-no-unsupported-directive-good"></span>

**正确示例**

替换后使用受支持的 `v-scope`、`v-effect`、`v-if`、`v-bind` 和 `v-on` 语法，不再依赖不支持的指令。

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) · [全部规则](all.md)

### `petite-vue/valid-v-effect`

要求 v-effect 具有非空表达式

[错误示例](#petite-vue-valid-v-effect-bad) · [正确示例](#petite-vue-valid-v-effect-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: 检测为 petite-vue 的 HTML 文档；普通 Vue SFC 不在此规则范围内。  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-effect": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-effect-bad"></span>

**错误示例**

每个 `v-effect` 都没有可执行表达式：值缺失、为空，或只包含空白。

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-effect></div>
<div v-effect=""></div>
<div v-effect="   "></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-effect-good"></span>

**正确示例**

两个 `v-effect` 值都包含表达式：一个更新 `el.textContent`，另一个递增 `count`。规则检查表达式非空，不检查副作用的业务逻辑。

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35) · [全部规则](all.md)

### `petite-vue/valid-v-scope`

要求 v-scope 绑定对象字面量

[错误示例](#petite-vue-valid-v-scope-bad) · [正确示例](#petite-vue-valid-v-scope-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: 检测为 petite-vue 的 HTML 文档；普通 Vue SFC 不在此规则范围内。  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-scope": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-scope-bad"></span>

**错误示例**

四个非空 `v-scope` 值分别是标识符、调用、算术表达式和数字；都不能解析为对象字面量。

```html annotate="remove:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope="count"></div>
<div v-scope="foo()"></div>
<div v-scope="a + b"></div>
<div v-scope="123"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-scope-good"></span>

**正确示例**

无值的 `v-scope` 使用根作用域。其他值是对象字面量，包括规则接受的加括号对象。

```html annotate="add:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope></div>
<div v-scope="{}"></div>
<div v-scope="{ count: 0 }"></div>
<div v-scope="({ count: 0 })"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46) · [全部规则](all.md)
