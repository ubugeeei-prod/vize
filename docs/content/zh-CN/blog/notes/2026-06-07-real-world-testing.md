---
title: 真实世界测试
description: Vize进入了真实世界测试阶段——真正的项目现在是测试套件，明确了通往v1.0.0的路线图。
---

<!-- Generated translation; source: blog/notes/2026-06-07-real-world-testing.md -->

# 真实世界测试

<div class="blog-post-meta">
  <span class="blog-meta-chip">
    <span>
      <span class="blog-meta-label">Published</span>
      <span class="blog-meta-value">2026-06-07</span>
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

<video class="blog-post-video" src="/blog/vize-real-world-testing.mp4" controls muted playsinline loop preload="metadata" aria-label="Real World Testing PV"></video>

Vize正进入一个新阶段。

直到现在，开发一直集中在实现功能、构建基础设施，
并通过专用测试套件和合成示例验证行为。

下一步就不同了。

我们现在正在积极寻找**现实世界的项目**来测试Vice。

## 目标

这一阶段的目标是发现兼容性缺口、规格缺口，
性能瓶颈和只出现在生产代码库中的边缘情况。

如果你维护着Vue应用、库、框架或工具，我们非常欢迎你
关于你用Vize运行它的经验。

每一份修复报告、复刻、基准测试结果和兼容性发现都有助于推动
该项目接近其首个稳定版本。

## 仍处于实验阶段——正确性优先

Vize仍应被视为实验性质。可能会有破坏性变更，修复是
仍然是预期中的，且在某些情况下行为可能与Vue不同。

这一阶段的重点不是功能开发。重点是正确性。
实际应用现在是测试套件。如果你遇到需要修复的问题，请务必
报告它——每一份报告都有助于改进编译器、语言规范，以及
整个生态系统。

## 如何帮助

我们正在等待大量的修复请求和永久居民通知。我们也在积极招募
大型Vue项目作为测试平台——代码库越大越真实，越丰富
宝贵的信号。如果你维护（或知道）一个相当规模的Vue应用、库，
框架或工具，请发起修复请求或联系我们，以便我们能对它运行 Vice。修复
欢迎发布报告、复制品和基准结果。

请参阅[测试与反馈](../../guide/testing.md)指南，了解如何检查输出
Playground，阅读现有的测试案例，与`vize check --profile`进行画像，并提出一个项目
作为端对端电子/VRT测试平台。

## v1.0.0路线图

当前阶段是**真实世界测试**。

一旦 Vize 成功完成这一阶段，项目将通过以下阶段推进：

- v1.0.0-alpha
- v1.0.0-beta
- v1.0.0-RC
- v1.0.0

Alpha、Beta和发布候选阶段将聚焦于稳定化和生态系统
兼容性、性能提升和长期维护保障。

目标不是急着升级到1.0版本。目标是赢得它。

如果你有兴趣帮助塑造Vize的未来，现在是获取的最佳时机
参与其中。
