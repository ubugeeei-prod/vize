---
title: "petite-vue/valid-v-scope"
description: "要求 v-scope 绑定对象字面量。"
---

# `petite-vue/valid-v-scope`

要求 v-scope 绑定对象字面量。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | 无 |

适用范围：检测为 petite-vue 的 HTML 文档；普通 Vue SFC 不在此规则范围内。

## 错误示例

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

## 正确示例

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

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "petite-vue/valid-v-scope": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [petite-vue 规则](../petite-vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46)
