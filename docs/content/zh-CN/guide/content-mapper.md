---
title: TypeScript Content Mapper
---

<!-- Generated translation; source: guide/content-mapper.md -->

# TypeScript Content Mapper

Content Mapper 是 TypeScript 用于检查编译器自身无法解析的文件类型的插件机制 ——
[TypeScript 7.1 API 路线图](https://github.com/microsoft/typescript-go/issues/4830)
将其定位为 Vue 所需的 TS Server 插件替代方案。该 API 已在
[microsoft/typescript-go#4712](https://github.com/microsoft/typescript-go/pull/4712)
中合并进 `typescript-go` 的 main 分支。

Vize 在 `vize` npm 包中内置了一个符合协议的 Content Mapper:支持 Content Mapper 的
`tsgo` 构建会启动 `vize content-mapper` 并直接检查 `.vue` 文件 —— 悬停、跳转定义、
重命名、补全和诊断全部映射回你编写的 SFC,无需再生成并行的 `.vue.ts` 项目。

> **⚠️ 预览:** Content Mapper 已合并到上游,但尚未包含在已发布的 TypeScript 7 platform
> packages 中。在包含该协议的版本发布之前,请从 `typescript-go` 的 main 分支构建支持
> Content Mapper 的 native TypeScript 二进制,并继续以 [`vize check`](./cli.md#check)
> 作为受支持的类型检查方式。

## 设置

安装 `vize` 并在 `tsconfig.json` 中声明映射器:

```bash
vp install -D vize
```

```json
{
  "compilerOptions": {
    "module": "preserve",
    "strict": true
  },
  "contentMappers": [
    {
      "package": "vize",
      "extensions": [".vue"]
    }
  ],
  "include": ["src"]
}
```

运行外部映射器进程需要显式选择加入:

```bash
tsgo --runExternalCode --noEmit -p tsconfig.json
```

在 VS Code 中,受信任的工作区里 Vize 扩展会自动向 TypeScript native preview 扩展注册
`.vue` 支持,同一个映射器也随之驱动编辑器。

## 选项

映射器条目接受一个 `options` 对象:

```json
{
  "contentMappers": [
    {
      "package": "vize",
      "extensions": [".vue"],
      "options": { "optionsApi": false }
    }
  ]
}
```

| 选项         | 默认值 | 用途                                          |
| ------------ | ------ | ---------------------------------------------- |
| `optionsApi` | `true` | 在模板中解析 Vue Options API 的实例绑定       |

无效的选项不会导致构建失败:Vize 会把它们作为定位到 tsconfig 内部的选项诊断
(`vize1`–`vize3`)报告,并以默认值继续。Vize 还声明了对项目 `noUnusedLocals`
编译器选项的依赖,因此 `<script setup>` 中未使用局部变量的报告遵循各项目自身的配置。

## 模板指令

`<script>` 块原样透传,因此 `@ts-expect-error` 可照常使用。模板表达式无法携带 TS 注释,
所以 Vize 通过协议映射 Vue 标准的 HTML 注释指令:

```vue
<template>
  <!-- @vue-expect-error -->
  {{ count.toFixed(true) }}

  <!-- @vue-ignore -->
  {{ untypedThirdPartyValue.field }}
</template>
```

- `<!-- @vue-expect-error -->` 抑制下一模板行的 TypeScript 诊断,若未抑制任何内容则
  报告 `vize4: Unused '@vue-expect-error' directive`。
- `<!-- @vue-ignore -->` 静默抑制。

若注释之后同一行还有内容,指令作用于该行的剩余部分;否则作用于下一个非空行。

## 协议

Vize 使用上游合并的 Content Mapper 协议 v1:UTF-8 位置编码、按项目的
`openProject`/`closeProject` 生命周期,以及能让 TypeScript 与内嵌 JSX 都正确解析的
`.tsx` 虚拟输出。一致性由 CI 保障:从固定的 `typescript-go` 修订版构建上游编译器,
并通过打包后的 npm 产物运行完整的 CLI、构建模式和 LSP 测试套件。

以 `vize` 为来源报告的诊断代码:

| 代码    | 含义                                       |
| ------- | ------------------------------------------ |
| `vize1` | 映射器选项的值不是对象                     |
| `vize2` | 未知的映射器选项                           |
| `vize3` | 映射器选项类型错误                         |
| `vize4` | 未使用的 `@vue-expect-error` 指令          |

## 限制

- 在 native preview 发布包含该 API 之前,需要从 `typescript-go` main 构建的 `tsgo`。
- 映射输入的 declaration map 依赖
  [microsoft/typescript-go#4860](https://github.com/microsoft/typescript-go/issues/4860)。
- 在上游 API 处于预览期间,生产环境的类型检查方式仍以 `vize check` 为受支持路径。
