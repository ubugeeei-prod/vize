---
title: "petite-vue/valid-v-effect"
description: "要求 v-effect 具有非空表达式。"
---

# `petite-vue/valid-v-effect`

要求 v-effect 具有非空表达式。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | 无 |

适用范围：检测为 petite-vue 的 HTML 文档；普通 Vue SFC 不在此规则范围内。

## 错误示例

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

## 正确示例

两个 `v-effect` 值都包含表达式：一个更新 `el.textContent`，另一个递增 `count`。规则检查表达式非空，不检查副作用的业务逻辑。

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
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
      "petite-vue/valid-v-effect": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [petite-vue 规则](../petite-vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35)
