---
title: "跨文件规则"
description: "需要查看多个文件的检查：父组件与子组件、provide 与 inject、路由与链接。"
---

# 跨文件规则

需要查看多个文件的检查：父组件与子组件、provide 与 inject、路由与链接。

在传给 `withVue` 的 `linter` 中设置 `crossFile: true`，或运行 `vize lint --cross-file` 来启用这些检查。诊断显示为 `vize:croquis/cf/<name>`；在 `linter.rules` 中去掉 `vize:`，写作 `croquis/cf/<name>`。

每个规则页面都有会触发诊断的示例、不会触发的示例，以及启用该规则的配置。

| 规则 | 检查内容 | 状态 |
| --- | --- | --- |
| <span id="ecosystem-vue-router-extra-param"></span>[`ecosystem/vue-router-extra-param`](./project/ecosystem-vue-router-extra-param.md) | 报告目标路由未声明的导航参数。 | 由 `vp run lint` 报告 |
| <span id="ecosystem-vue-router-missing-param"></span>[`ecosystem/vue-router-missing-param`](./project/ecosystem-vue-router-missing-param.md) | 报告具名导航省略了必需的路由参数。 | 由 `vp run lint` 报告 |
| <span id="ecosystem-vue-router-param-type"></span>[`ecosystem/vue-router-param-type`](./project/ecosystem-vue-router-param-type.md) | 报告路由参数值不被所声明的路径段接受。 | 由 `vp run lint` 报告 |
| <span id="ecosystem-vue-router-unknown-route"></span>[`ecosystem/vue-router-unknown-route`](./project/ecosystem-vue-router-unknown-route.md) | 报告已安装路由器未声明的导航名称。 | 由 `vp run lint` 报告 |
| <span id="html-cross-component-nesting"></span>[`html/cross-component-nesting`](./project/html-cross-component-nesting.md) | 检查导入组件组合后的实际 HTML 嵌套。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-array-mutation"></span>[`vize:croquis/cf/array-mutation`](./project/vize-croquis-cf-array-mutation.md) | 在 Vue 2.7 中，使用可通知更新的操作替换数组索引处的值。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-async-boundary"></span>[`vize:croquis/cf/async-boundary`](./project/vize-croquis-cf-async-boundary.md) | 响应式状态跨越异步边界，可能被观察为过期值。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-async-no-suspense"></span>[`vize:croquis/cf/async-no-suspense`](./project/vize-croquis-cf-async-no-suspense.md) | 异步组件在没有 Suspense 边界的情况下渲染。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-browser-api-ssr"></span>[`vize:croquis/cf/browser-api-ssr`](./project/vize-croquis-cf-browser-api-ssr.md) | 在组件可能于服务端渲染的位置使用了浏览器专用 API。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-circular-dep"></span>[`vize:croquis/cf/circular-dep`](./project/vize-croquis-cf-circular-dep.md) | 组件相互导入，形成循环。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-circular-reactive-dependency"></span>[`vize:croquis/cf/circular-reactive-dependency`](./project/vize-croquis-cf-circular-reactive-dependency.md) | 响应式计算相互依赖，形成循环。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-closure-captures-reactive"></span>[`vize:croquis/cf/closure-captures-reactive`](./project/vize-croquis-cf-closure-captures-reactive.md) | 闭包捕获了响应式值，无法看到后续更新。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-composable-outside-setup"></span>[`vize:croquis/cf/composable-outside-setup`](./project/vize-croquis-cf-composable-outside-setup.md) | 组合式函数在 `setup` 外被调用。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-computed-side-effects"></span>[`vize:croquis/cf/computed-side-effects`](./project/vize-croquis-cf-computed-side-effects.md) | 计算 getter 写入状态或执行其他副作用。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-deep-import"></span>[`vize:croquis/cf/deep-import`](./project/vize-croquis-cf-deep-import.md) | 导入链超过项目允许的深度。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-destructuring-breaks-reactivity"></span>[`vize:croquis/cf/destructuring-breaks-reactivity`](./project/vize-croquis-cf-destructuring-breaks-reactivity.md) | 解构响应式对象会复制字段并丢失跟踪。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-di-outside-setup"></span>[`vize:croquis/cf/di-outside-setup`](./project/vize-croquis-cf-di-outside-setup.md) | `provide` 或 `inject` 在 `setup` 外被调用。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-dom-access-without-next-tick"></span>[`vize:croquis/cf/dom-access-without-next-tick`](./project/vize-croquis-cf-dom-access-without-next-tick.md) | 在 Vue 完成更新刷新之前读取 DOM。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-duplicate-id"></span>[`vize:croquis/cf/duplicate-id`](./project/vize-croquis-cf-duplicate-id.md) | 多个组件使用同一个元素 id。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-event-listener-leak"></span>[`vize:croquis/cf/event-listener-leak`](./project/vize-croquis-cf-event-listener-leak.md) | 注册了事件监听器，却从未移除。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-event-modifier"></span>[`vize:croquis/cf/event-modifier`](./project/vize-croquis-cf-event-modifier.md) | 事件监听器使用了发出事件不支持的修饰符。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-hydration-risk"></span>[`vize:croquis/cf/hydration-risk`](./project/vize-croquis-cf-hydration-risk.md) | 服务器与客户端的渲染结果可能不一致。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-inherit-attrs-unused"></span>[`vize:croquis/cf/inherit-attrs-unused`](./project/vize-croquis-cf-inherit-attrs-unused.md) | 设置了 `inheritAttrs: false`，但组件从不读取属性。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-inject-without-symbol"></span>[`vize:croquis/cf/inject-without-symbol`](./project/vize-croquis-cf-inject-without-symbol.md) | `inject` 使用普通键，而不是 `InjectionKey` symbol。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-injected-async-mutation-race"></span>[`vize:croquis/cf/injected-async-mutation-race`](./project/vize-croquis-cf-injected-async-mutation-race.md) | 注入值被可能发生竞态的异步任务修改。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-lifecycle-outside-setup"></span>[`vize:croquis/cf/lifecycle-outside-setup`](./project/vize-croquis-cf-lifecycle-outside-setup.md) | 生命周期钩子在 `setup` 外注册。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-lifecycle-without-cleanup"></span>[`vize:croquis/cf/lifecycle-without-cleanup`](./project/vize-croquis-cf-lifecycle-without-cleanup.md) | 生命周期钩子启动任务，却从不清理。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-missing-required-prop"></span>[`vize:croquis/cf/missing-required-prop`](./project/vize-croquis-cf-missing-required-prop.md) | 未传入必需的 prop。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-missing-suspense"></span>[`vize:croquis/cf/missing-suspense`](./project/vize-croquis-cf-missing-suspense.md) | 异步依赖在 Suspense 边界之外使用。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-module-scope-reactive"></span>[`vize:croquis/cf/module-scope-reactive`](./project/vize-croquis-cf-module-scope-reactive.md) | 响应式状态在模块作用域创建，被所有调用者共享。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-multi-root-attrs"></span>[`vize:croquis/cf/multi-root-attrs`](./project/vize-croquis-cf-multi-root-attrs.md) | 多根节点组件接收了属性，却没有放置目标。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-mutated-after-escape"></span>[`vize:croquis/cf/mutated-after-escape`](./project/vize-croquis-cf-mutated-after-escape.md) | 响应式对象逸出其所有者后被修改。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-non-reactive-provide"></span>[`vize:croquis/cf/non-reactive-provide`](./project/vize-croquis-cf-non-reactive-provide.md) | 提供的值不具响应性，后代无法看到更新。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-non-unique-id"></span>[`vize:croquis/cf/non-unique-id`](./project/vize-croquis-cf-non-unique-id.md) | 循环内的元素 id 对每个列表项并不唯一。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-object-identity-comparison"></span>[`vize:croquis/cf/object-identity-comparison`](./project/vize-croquis-cf-object-identity-comparison.md) | 用稳定的记录 ID 比较原始对象与响应式代理所表示的记录。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-pinia-getter"></span>[`vize:croquis/cf/pinia-getter`](./project/vize-croquis-cf-pinia-getter.md) | 避免在 setup 中将 Pinia getter 的值复制为不再跟随更新的快照。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-prop-type-mismatch"></span>[`vize:croquis/cf/prop-type-mismatch`](./project/vize-croquis-cf-prop-type-mismatch.md) | 传入的 prop 值与声明类型不匹配。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-provide-inject-type"></span>[`vize:croquis/cf/provide-inject-type`](./project/vize-croquis-cf-provide-inject-type.md) | 提供值与其 inject 类型不一致。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-provide-without-symbol"></span>[`vize:croquis/cf/provide-without-symbol`](./project/vize-croquis-cf-provide-without-symbol.md) | `provide` 使用普通键，而不是 `InjectionKey` symbol。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-reactive-export"></span>[`vize:croquis/cf/reactive-export`](./project/vize-croquis-cf-reactive-export.md) | 响应式状态从模块导出。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-reactivity-outside-setup"></span>[`vize:croquis/cf/reactivity-outside-setup`](./project/vize-croquis-cf-reactivity-outside-setup.md) | 响应式 API 在 `setup` 外被调用。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-reassignment-breaks-reactivity"></span>[`vize:croquis/cf/reassignment-breaks-reactivity`](./project/vize-croquis-cf-reassignment-breaks-reactivity.md) | 重新赋值响应式绑定，将其替换为普通值。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-reference-escapes-scope"></span>[`vize:croquis/cf/reference-escapes-scope`](./project/vize-croquis-cf-reference-escapes-scope.md) | 响应式引用逸出拥有其生命周期的作用域。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-setup-context-violation"></span>[`vize:croquis/cf/setup-context-violation`](./project/vize-croquis-cf-setup-context-violation.md) | 在每个实例的 setup 中创建应由组件实例独立持有的响应式状态。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-shallow-deep-access"></span>[`vize:croquis/cf/shallow-deep-access`](./project/vize-croquis-cf-shallow-deep-access.md) | 读取 `shallowReactive` 或 `shallowRef` 的深层属性，却假定它受到跟踪。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-spread-breaks-reactivity"></span>[`vize:croquis/cf/spread-breaks-reactivity`](./project/vize-croquis-cf-spread-breaks-reactivity.md) | 展开响应式对象会复制值并丢失跟踪。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-suspense-no-fallback"></span>[`vize:croquis/cf/suspense-no-fallback`](./project/vize-croquis-cf-suspense-no-fallback.md) | `<Suspense>` 没有后备内容。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-template-ref-timing"></span>[`vize:croquis/cf/template-ref-timing`](./project/vize-croquis-cf-template-ref-timing.md) | 模板 ref 在组件挂载之前被读取。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-toraw-mutation"></span>[`vize:croquis/cf/toraw-mutation`](./project/vize-croquis-cf-toraw-mutation.md) | 使用 `toRaw` 后修改了原始对象。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-uncaught-error"></span>[`vize:croquis/cf/uncaught-error`](./project/vize-croquis-cf-uncaught-error.md) | 组件可能抛错，却没有错误边界捕获。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-undeclared-emit"></span>[`vize:croquis/cf/undeclared-emit`](./project/vize-croquis-cf-undeclared-emit.md) | 组件发出了未声明的事件。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-undeclared-prop"></span>[`vize:croquis/cf/undeclared-prop`](./project/vize-croquis-cf-undeclared-prop.md) | 父组件传入了子组件未声明的 prop。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-undefined-slot"></span>[`vize:croquis/cf/undefined-slot`](./project/vize-croquis-cf-undefined-slot.md) | 父组件填充了子组件未暴露的插槽。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-unhandled-event"></span>[`vize:croquis/cf/unhandled-event`](./project/vize-croquis-cf-unhandled-event.md) | 子组件发出的事件没有父组件处理。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-unmatched-inject"></span>[`vize:croquis/cf/unmatched-inject`](./project/vize-croquis-cf-unmatched-inject.md) | `inject` 指定的键没有任何祖先提供。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-unmatched-listener"></span>[`vize:croquis/cf/unmatched-listener`](./project/vize-croquis-cf-unmatched-listener.md) | 父组件监听了子组件不发出的事件。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-unregistered-component"></span>[`vize:croquis/cf/unregistered-component`](./project/vize-croquis-cf-unregistered-component.md) | 模板使用了未注册或导入的组件。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-unresolved-import"></span>[`vize:croquis/cf/unresolved-import`](./project/vize-croquis-cf-unresolved-import.md) | 导入无法解析为模块。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-unused-attrs"></span>[`vize:croquis/cf/unused-attrs`](./project/vize-croquis-cf-unused-attrs.md) | 透传属性传给了未使用它们的多根节点组件。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-unused-emit"></span>[`vize:croquis/cf/unused-emit`](./project/vize-croquis-cf-unused-emit.md) | 已声明的 emit 从未使用。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-unused-provide"></span>[`vize:croquis/cf/unused-provide`](./project/vize-croquis-cf-unused-provide.md) | 提供的键从未被注入。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-value-extraction-breaks-reactivity"></span>[`vize:croquis/cf/value-extraction-breaks-reactivity`](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md) | 将响应式值读取到局部变量后丢失后续更新。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-watch-can-be-computed"></span>[`vize:croquis/cf/watch-can-be-computed`](./project/vize-croquis-cf-watch-can-be-computed.md) | 侦听器只将值复制到状态，可以改为计算属性。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-watcheffect-async"></span>[`vize:croquis/cf/watcheffect-async`](./project/vize-croquis-cf-watcheffect-async.md) | 为异步 watchEffect 处理失效清理，避免写入过期响应。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-watcher-outside-setup"></span>[`vize:croquis/cf/watcher-outside-setup`](./project/vize-croquis-cf-watcher-outside-setup.md) | `watch` 或 `watchEffect` 在 `setup` 外被调用。 | 计划中；尚未报告 |
| <span id="vue-cross-file-attrs-fallthrough"></span>[`vue/cross-file-attrs-fallthrough`](./project/vue-cross-file-attrs-fallthrough.md) | 父组件传入属性，但已解析子组件的根无法继承它们，也没有显式使用 $attrs。 | 由 `vp run lint` 报告 |

[规则概览](../rules/index.md) · [规则选项](/rules/options.md) · [从 ESLint 迁移](/rules/migration.md)
