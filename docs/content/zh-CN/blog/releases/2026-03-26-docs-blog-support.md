---
title: 文档博客
description: Vize文档现在既可以托管发布说明，也可以承载不规则说明。
---

<!-- Generated translation; source: blog/releases/2026-03-26-docs-blog-support.md -->

# 医生博客

<div class="blog-post-meta">
  <span class="blog-meta-chip">
    <span>
      <span class="blog-meta-label">Published</span>
      <span class="blog-meta-value">2026-03-26</span>
    </span>
  </span>
  <a class="blog-author-card" href="https://github.com/ubugeeei">
    <img src="https://github.com/ubugeeei.png" alt="ubugeeei" />
    <span class="blog-author-text">
      <span class="blog-meta-label">Author</span>
      <span class="blog-meta-value">ubugeeei</span>
    </span>
  </a>
</div>

Vize文档现在可以在`docs/content/blog/`内部直接托管两种帖子：

- `releases/` 用于发布变更和发布沟通
- `notes/`用于不规则写作，如开发日志、架构说明和项目更新

## 发生了什么变化

- 在文档中增加了顶层的**博客**板块。
- 将写作流程拆分为**发布说明**和**备注**。
- 增加了起始模板，使未来帖子更容易创建并保持一致性。

## 为什么这很重要

Vize已经发展成不仅仅是一个包的README。有些更新应放在参考文档中，但有些则需要叙述背景：发布了什么、为何重要、哪些仍处于实验阶段，以及项目的发展方向。

这种新的博客结构创造了这样的空间，而无需引入单独的网站或第二种发布流程。

## 写作地点

- 发布帖子：`docs/content/blog/releases/`
- 不规则岗位：`docs/content/blog/notes/`
- 模板：`docs/templates/blog-release.md`和`docs/templates/blog-note.md`
