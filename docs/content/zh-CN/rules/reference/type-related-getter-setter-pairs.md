---
title: "type/related-getter-setter-pairs"
description: "要求成对的 getter 和 setter 使用兼容的类型。"
---

# `type/related-getter-setter-pairs`

要求成对的 getter 和 setter 使用兼容的类型。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

getter 返回 string，setter 却接受 number，因此这一对类型不兼容。

```vue annotate="remove:4"
<script setup lang="ts">
interface Box {
  get size(): string;
  set size(value: number);
}
</script>
```

## 正确示例

getter 和 setter 都使用 string。

```vue annotate="add:4"
<script setup lang="ts">
interface Box {
  get size(): string;
  set size(value: string);
}
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/related-getter-setter-pairs": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/related_getter_setter_pairs.rs#L5)
