---
title: MCP服务器
---

<!-- Generated translation; source: integrations/mcp.md -->

# MCP服务器

> **⚠️ 正在开发中：**Vize正在积极开发中，尚未准备好投入生产使用。MCP服务器功能可能会在未通知的情况下发生变化。

Vize为AI驱动的开发工作流提供了[模型上下文协议（MCP）](https://modelcontextprotocol.io/)服务器。MCP服务器连接了你的组件库（Musea）与AI助手之间的差距，使它们能够理解、导航并操作你的Vue组件。

## 安装

从[Vite+安装指南](https://viteplus.dev/guide/install)中安装一次`vp`，然后将服务器添加到你的项目中：

```bash
vp install -D @vizejs/musea-mcp-server
```

## 什么是MCP？

模型上下文协议是一个开放标准，用于连接人工智能助手（如Claude、ChatGPT等）与开发工具。MCP不再是AI助手猜测你的代码库，而是提供对真实组件数据的结构化访问——道具、事件、槽位、变体和文档。

Vize的MCP服务器会从Musea画廊中获取组件信息，因此你的AI助手对组件的理解与浏览画廊的开发者拥有相同的理解。

## 能力

MCP 服务器为 AI 助手提供以下工具：

### 组件发现

- **列出所有组件**— 浏览所有注册组件及其类别、标签和状态
- **搜索组件**— 通过名称、标签或描述查找组件
- **获取组件元数据**— 检索特定组件的详细信息

### 组件 API

- **Props**— 完整的道具定义，包含类型、默认值和所需状态
- **事件**— 具有有效载荷类型的发射事件
- **槽槽**— 带有槽道具类型的命名槽
- **Expose**— 公开暴露的方法和属性

### 故事信息

- **变体列表**— 所有在艺术文件中定义的变体
- **变体源代码**— 每个变体的模板代码
- **默认变体**— 默认显示的变体

### 设计代币

- **令牌列表**— 所有来自令牌文件的设计令牌
- **令牌类别**— 颜色、排版、间距、断点
- **令牌解析**— 将语义令牌解析为其原始值

## 布置

### 与克劳德·代码

将MCP服务器添加到你的Claude Code配置中：

```json
// .claude/settings.json
{
  "mcpServers": {
    "vize-musea": {
      "command": "vp",
      "args": ["dlx", "@vizejs/musea-mcp-server"]
    }
  }
}
```

### 使用Claude桌面

添加到你的Claude桌面MCP配置中：

```json
{
  "mcpServers": {
    "vize-musea": {
      "command": "vp",
      "args": ["dlx", "@vizejs/musea-mcp-server"]
    }
  }
}
```

### 与其他AI助手一起

任何兼容MCP的AI助手都可以使用该服务器。配置模式相同——将助理指向`vp dlx @vizejs/musea-mcp-server`。

## 使用场景

### 组件发现

请你的AI助手找到合适的组件：

> “我们有哪些按钮组件？给我看看VFButton的变体。”

AI可以查询MCP服务器，查找所有与按钮相关的组件、它们的道具和可用变体——然后建议正确的使用方式。

### 代码生成

用正确的道具生成组件使用：

> “使用我们的VFInput和VFTextarea组件创建表单，包括验证错误状态。”

AI能够准确知道MCP服务器上的道具名称、类型和可用变体，能够生成准确的代码，而无需产生道具名称的幻觉。

### API 参考

程序性查询组件API：

> “VFNameBadgePreview接受哪些道具？用户角色的有效值是什么？”

AI会返回你代码库中的真实道具定义，而不是通用的猜测。

### 文档协助

> “根据我们的SponsorGrid组件的道具和变体编写文档。”

AI可以通过MCP检查实际组件元数据，生成准确的文档。

## 工作原理

```
AI Assistant
  ↕ MCP Protocol (JSON-RPC over stdio)
@vizejs/musea-mcp-server
  ↕ Reads art files and component sources
Your Project (*.art.vue files + components)
```

MCP服务器：

1. 发现你项目中的所有`*.art.vue`文件
2. 利用`vize_musea`解析组件元数据
3. 通过MCP工具暴露元数据
4. 实时响应AI助手的询问
