---
title: 编译器检查器
---

<!-- Generated translation; source: guide/compiler-inspector.md -->

# 编译检查员

操场检查员收集用于审查一`.vue`所需的编译器和分析面
复刻版。它显示官方的Vue SFC编译器输出、Vize编译器输出、虚拟TS、VIR以及
本地批次的跨文件图。

从操场打开检查器：

```bash
https://vizejs.dev/play/?tab=inspector
```

检查员在浏览器中执行以下检查：

- `@vue/compiler-sfc` 用于参考输出
- Vize WASM 用于 Vize 输出
- 针对所选文件的Canon支持的虚拟技术系统
- Croquis VIR 用于所选文件
- 与CLI共享的原生`vize_curator`图和微分元数据
- 有效载荷文件的跨文件诊断
- DOM或SSR目标选择
- 可选自定义渲染器和模板语法模式控制
- 两个编译器的完整输出标签页
- 一个包含仅Vue和仅Vize行的比较标签页
- 永久链接和预填充的拉取请求链接

## CLI有效载荷

当复制品已经存在于本地项目时，使用`vize inspector`。单个文件会产生
Playground 默认网址：

```bash
vize inspector src/App.vue
```

目录和团块生成批量有效载荷。游乐场会打开批次，让你切换
文件之间。

```bash
vize inspector src/components
vize inspector "src/**/*.vue" --target ssr
```

对于大批量，应发布 JSON 代替长 URL：

```bash
vize inspector "src/**/*.vue" --format json --output inspector-payload.json
```

对于AI代理或终端切换，发送代理报告。它包括payload、playground的URL，
汇总指标和跨文件图元数据。

```bash
vize inspector "src/**/*.vue" --format agent --output inspector-agent.json
```

在本地开发检查中，CLI也可以直接运行编译器比较。该方法的用途
当前二进制文件中的Rust编译器，并从当前项目加载`@vue/compiler-sfc`
Vize工作空间`node_modules`。

```bash
vize inspector "src/**/*.vue" --format compare --output inspector-compare.json
```

有效载荷和代理报告由`vize_curator`生成，这也是本地仅使用的一个Rust箱
通过游乐场WASM绑定用于图和线差分元数据。这样可以保留批处理CLI报告，
浏览器检查对齐，同时官方 Vue 编译器仍运行于浏览器内。

实用选项：

| 选项                | 描述                                         |
| ------------------- | -------------------------------------------- |
| `--target dom`      | 比较 VDOM 编译器输出                         |
| `--target ssr`      | 比较SSR编译器输出                            |
| `--format agent`    | Emit agent-readable JSON with graph metadata |
| `--format compare`  | 运行仅开发者CLI与Vue                         |
| `--custom-renderer` | 在游乐场启用自定义渲染模式                   |
| `--template-syntax` | 选择`standard`、`strict`或`quirks`           |
| `--max-files <n>`   | 限制批处理有效载荷中的文件数量               |
| `--playground-url`  | 覆盖用于链接的游乐场网址                     |

## 公关工作流程

打开编译器奇偶校验PR时，在PR正体中包含检查器永久链接，并添加
最小的夹具或完整快照，使输出变更可在CI中审查。预填的PR
Link只是一个起点;推送分支后，如果GitHub要求更换比较首脑。

有用的公关证据如下：

- 检查员永久链接
- 选定目标和选项
- 最小化`.vue`夹具或完整快照
- 当修复跨越编译器表面时，相关的虚拟TS、VIR或图上下文
- Vize输出应与Vue一致或有意不同的原因
- 覆盖被触及编译器表面的本地验证命令
