---
title: 贡献
---

<!-- Generated translation; source: contributing.md -->

# 贡献

感谢你帮忙让Vize更锋利。该项目正处于**真实世界测试**阶段，正在推进中
进入v1 alpha阶段，所以小而聚焦且有明确验证的改动最容易复习。如果你
我们来这里是来报告发现，而不是开设永久居民，首先是
[测试与反馈](./guide/testing.md)向导。

## 布置

用`.node-version`的Node.js版本和`rust-toolchain.toml`的Rust版本。该
workspace 在 `Cargo.toml` 中声明了最低支持的 Rust 版本（MSRV）的 `1.98.0`
（`[workspace.package].rust-version`）;贡献必须按该版本编译。

默认的 Nix shell 包含可重复的本地工具链。Blacksmith测试盒支持是
可选，并且与置顶的 Blacksmith CLI、`rsync` 和 GitHub CLI 一起独立于 shell 中：

```sh
nix develop             # local development
nix develop .#testbox   # hosted Testbox workflows
```

从工作区根安装依赖：

```sh
vp install --frozen-lockfile --prefer-offline
```

如果`vp`还没用，请先安装 [Vite+](https://viteplus.dev/guide/install)。

## 常见检查

先用最窄的范围覆盖你的变化，然后在涉及共享行为时再扩大范围。

```sh
vp check <changed-files>
node --test tests/tooling/<test-file>.test.ts
cargo fmt --all -- --check
cargo test -p <crate>
```

在打开更改共享工具、发布自动化、原生绑定或编译器的PR之前
行为，在实际可行的情况下，从CI中本地运行相关工作空间任务。

根构建、测试和 Lint 工作流程默认是本地的，不需要托管凭证：

```sh
vp run --workspace-root build
vp run --workspace-root test
vp run --workspace-root lint
```

在 Nix 的开发框架内，`vp build`、`vp test` 和 `vp lint` 是这些的简写
工作区任务。

对于单命令的Linux CI奇偶校验，需要专用的Testbox shell。默认的`nix develop`壳
故意省略Blacksmith，且不需要其托管的工件或凭证：

```sh
nix develop .#testbox
```

然后运行下面的守护生命周期。它会在预热前清除任何旧的盒子ID，跳过远程任务
认证、推送或预热都失败，且总是试图阻止成功预热的盒子
任务失败时：

```sh
run_testbox_checks() {
  unset BLACKSMITH_TESTBOX_ID testbox_output
  "$VIZE_BLACKSMITH_BIN" auth login || return
  git push --set-upstream origin "$(git branch --show-current)" || return

  if testbox_output="$(vp run --workspace-root testbox:warmup)"; then
    BLACKSMITH_TESTBOX_ID="$(printf '%s\n' "$testbox_output" | tail -n1)"
  else
    warmup_status=$?
    unset testbox_output
    return "$warmup_status"
  fi
  if [ -z "$BLACKSMITH_TESTBOX_ID" ]; then
    printf '%s\n' "Testbox warmup returned no box id." >&2
    unset BLACKSMITH_TESTBOX_ID testbox_output
    return 1
  fi
  export BLACKSMITH_TESTBOX_ID

  if vp run --workspace-root build:testbox &&
    vp run --workspace-root test:testbox &&
    vp run --workspace-root lint:testbox; then
    testbox_status=0
  else
    testbox_status=$?
  fi
  if vp run --workspace-root testbox:stop; then
    stop_status=0
  else
    stop_status=$?
  fi
  unset BLACKSMITH_TESTBOX_ID testbox_output

  if [ "$testbox_status" -ne 0 ]; then
    return "$testbox_status"
  fi
  return "$stop_status"
}
run_testbox_checks
```

对于GitHub Actions的变更，请使用`actrun`在推送前对工作流程图进行lint或预览：

```sh
vp run actrun:lint
vp run actrun:dry-run
vp run actrun:job --job check-js
```

对于Blacksmith Testbox的工作变更，也要验证工作流程形状，用
`node --test tests/tooling/github-workflows.test.ts`。

## 语言处理器变革学科

Vize 遵循了 rustc、TypeScript、TypeScript-Go 和 Flow 的编译器-项目实践：对
修改，添加最小有意义的夹具，作为合同审查生成的输出，然后扩展为
当接触面需要时，使用奇偶校验、性能或释放门。参见
[语言工程实践](./architecture/language-engineering-practices.md)完整版
《黑客帝国》。

在PR中，使用以下变更类（如适用）：

- 解析器或 AST
- 编译器和代码生成
- 语义分析、棉絮和跨文件分析
- 虚拟TypeScript和类型检查
- Formatter 和 LSP
- 运行时打包、发布或文档

对于面向语言的变更，包含证明行为的夹具或快照差异。对于
快照刷新，解释为什么新输出正确，并避免广泛的基线流失，除非
公关专注于这个输出家族。

当编译器不匹配源自外部复制或本地项目文件时，使用游乐场
[编译检查员](./guide/compiler-inspector.md)检查官方Vue输出，Vice输出，
虚拟TS、VIR和跨文件图。将检查员永久链接添加到PR主体，然后
最小化的夹具或完整快照，将输出转化为审查合同。本地批次可以
与`vize inspector <file-or-glob>`打包，代理切换可以使用
`vize inspector --format agent`。

## 拉取请求

- 使用常规提交来处理提交消息和PR标题，例如：
  `fix(vite-plugin): surface SFC compile errors`。
- 让PR专注于一项行为改变或一项文档/治理变更。
- 在PR正体中包含验证命令。
- 除非PR专门针对这些输出，否则不要刷新大型快照基线。
- 不包含秘密、注册令牌、私有漏洞细节或机器本地路径
  报告、提交或PR。

## 修正请求

使用修复报告模板来处理回归、崩溃、错误诊断、软件包安装
问题和发布失败。使用功能请求模板来处理新的集成、API 变更，
或者改进工作流程。一个极简的复制品——理想情况下是游乐场检查员的链接——会形成一个
报告更快，行动起来更快。

安全报告应随后发布
[`SECURITY.md`](https://github.com/ubugeeei-prod/vize/blob/main/SECURITY.md)而不是公众
修正模板。

## 行为准则与治理准则

参与即表示您同意遵守
[贡献者 Covenant v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/)。该
治理模型和决策过程有文档
[`GOVERNANCE.md`](https://github.com/ubugeeei-prod/vize/blob/main/GOVERNANCE.md)。求助于寻找
右声道，见[`SUPPORT.md`](https://github.com/ubugeeei-prod/vize/blob/main/SUPPORT.md)。
