export const chineseShared: Record<string, string> = {
  "Shared project files": "共享项目文件",
  "Example qualification": "示例验证状态",
  "Public explanation": "公开说明",
  Producer: "诊断实现",
  "Cross-file index": "跨文件规则索引",
  "Experimental Rust analyzer; not an individual CLI code":
    "实验性 Rust 分析器；CLI 不会单独发出此代码",
  "Contract; no current producer": "诊断约定；目前没有实现",
  "Project-specific lint ID": "项目专属 lint ID",
  "The public CLI exposes the same pass with `vize lint --cross-file`. Displayed `vize:croquis/cf/*` codes use `croquis/cf/*` in `lint.vize.rules` (omit `vize:`). Information/hint diagnostics become CLI warnings. Related locations explain the source/consumer relationship.":
    "公共 CLI 可通过 `vize lint --cross-file` 执行同一检查。显示的 `vize:croquis/cf/*` 代码在 `lint.vize.rules` 中写为 `croquis/cf/*`（省略 `vize:`）。信息和提示诊断会转换为 CLI 警告。相关位置说明来源与消费者之间的关系。",
  "Not emitted": "不产生诊断",
  "context-dependent": "依上下文而定",
  None: "无",
  info: "info",
  "error / warning (with default)": "error / warning（具有默认值时）",
  "None; review related files and apply the repair": "无；请检查相关文件并应用修复",
  "Analyzed component graph and the supported facts described below":
    "已分析的组件图，以及下方说明的受支持事实",
  "CSS inside SFC style blocks": "SFC style 块内的 CSS",
  "HTML documents detected as petite-vue; ordinary Vue SFCs are outside this rule's scope.":
    "检测为 petite-vue 的 HTML 文档；普通 Vue SFC 不在此规则范围内。",
  "JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form":
    "Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式",
  "Musea .art.vue art, variant, and style blocks": "Musea .art.vue 的 art、variant 和 style 块",
  "Nuxt configuration files (nuxt.config.ts)": "Nuxt 配置文件（nuxt.config.ts）",
  "Reachable project declarations and imported components": "可达的项目声明及导入组件",
  "Type information in Vue SFC scripts and templates, for the constructs shown below":
    "Vue SFC 脚本和模板中下方所示结构的类型信息",
  "Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts":
    "面向 Vapor 的脚本检查；显式启用后，限制也适用于普通脚本",
  "No per-code options; supported CLI findings accept severity overrides":
    "无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度",
  "crossFile; rule severity (off/warn/error)": "crossFile；规则严重程度（off/warn/error）",
  "A single SFC root link may inherit its target from parent attributes. This example uses a nested link, whose target must be explicit.":
    "SFC 的唯一根链接可能从父组件属性继承目标。此示例使用嵌套链接，必须显式提供目标。",
  "Checks non-interactive elements without an interactive role. Native buttons and elements with an interactive ARIA role are outside this rule's finding.":
    "检查没有交互角色的非交互式元素。原生按钮和具有交互式 ARIA 角色的元素不在此规则诊断范围内。",
  "Enable typeAware and this rule explicitly. The default disallows nullable numbers, while non-null numbers are allowed.":
    "显式启用 typeAware 和此规则。默认不允许可空数字，允许非空数字。",
  "Historical Vue 2.7 only: use matching Vue 2.7 and SFC compiler dependencies for this scenario. Vue 3 proxies track array index assignment, so `items[0] = next` is reactive in Vue 3 and is not a Vue 3 defect. This published code has no current producer.":
    "仅适用于历史 Vue 2.7：此情形须使用匹配的 Vue 2.7 和 SFC 编译器依赖。Vue 3 代理跟踪数组索引赋值，因此 `items[0] = next` 在 Vue 3 中具有响应性，并非 Vue 3 缺陷。此公开诊断代码目前没有实现。",
  "Intentional shared application stores may export reactive state. This scenario requires isolated state and does not claim every reactive export is invalid. No current producer emits this contract.":
    "有意共享的应用 store 可以导出响应式状态。此情形要求状态隔离，并未声称所有响应式导出都无效。目前没有实现发出此诊断约定。",
  "Module-scope reactive state is legal for intentional application stores. This example assumes component/request isolation; the published contract currently has no producer.":
    "有意共享的应用 store 在模块作用域创建响应式状态是合法的。此示例假定组件与请求隔离；此公开诊断约定目前没有实现。",
  "Module-scope watchers are valid when their owner keeps and calls a stop handle or intentionally gives them application lifetime. This example requires component-owned lifetimes; the contract has no current producer.":
    "所有者保留并调用停止句柄，或有意赋予应用级生命周期时，模块作用域侦听器是有效的。此示例要求组件拥有生命周期；该诊断约定目前没有实现。",
  "Pinia must be installed, and main.ts installs its plugin before mounting. Reading `store.doubled` directly inside a tracked computation or template is valid; the defect here is taking a plain snapshot. This contract currently has no producer.":
    "必须安装 Pinia，且 main.ts 须在挂载前安装其插件。在受跟踪的计算或模板内直接读取 `store.doubled` 是有效的；此处问题是取普通快照。该诊断约定目前没有实现。",
  "Refs may legitimately be returned from composables or shared across scopes. This example explicitly requires a snapshot cache; it does not claim that unmount invalidates a ref. No current producer emits this contract.":
    "Refs 可以合法地从组合式函数返回或跨作用域共享。此示例显式要求快照缓存，并未声称卸载会使 ref 失效。目前没有实现发出此诊断约定。",
  "Requires an .art.vue file and the token inventory shown below. It does not infer a token from an arbitrary color.":
    "需要 .art.vue 文件及下方所示的令牌清单。不会从任意颜色推断设计令牌。",
  "Suspense without a fallback is valid Vue syntax. This is a chosen loading-UI convention, not a compiler error; the published contract has no current producer.":
    "没有后备内容的 Suspense 是有效的 Vue 语法。这是选定的加载界面规范，并非编译器错误；此公开诊断约定目前没有实现。",
  "The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.":
    "正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。",
  "The boundary producer reads macros.is_async(), but source parsing currently records top-level await on the script-setup scope instead. The complete Bad/Good source pair below therefore produces no async-no-suspense finding through the current CLI. It explains the Suspense convention; supplying the missing macro fact is implementation follow-up work.":
    "边界诊断实现读取 macros.is_async()，但当前源代码解析将顶层 await 记录在 script-setup 作用域。因此，下方完整的错误与正确源代码对照不会通过当前 CLI 产生 async-no-suspense 诊断。它解释 Suspense 规范；补充缺失宏事实属于后续实现工作。",
  "The complete Vue project below illustrates update feedback and its repair. It is not a qualified CLI finding witness: the diagnostic producer requires retained reactive-flow reference identities and edges, as shown by the accompanying graph. These sources do not establish that the current source path will emit this exact code. Dedicated tracked-ID graph finding controls remain separate from source grammar checks.":
    "下方完整 Vue 项目展示更新反馈及其修复，但不构成已验证的 CLI 诊断复现：诊断实现需要附图所示的保留响应式流引用身份和边。这些源代码不能证明当前源代码路径会发出此确切代码。专门的跟踪 ID 图诊断对照仍与源代码语法检查分开。",
  "The complete installed router must be reachable from the application's createApp(...).use(router). Unknown/dynamic route tables do not prove unknown-name findings. Missing params are warnings because navigation may inherit a value from the current route.":
    "完整且已安装的路由器必须从应用的 createApp(...).use(router) 可达。未知或动态路由表不能证明未知名称诊断。缺失参数属于警告，因为导航可能从当前路由继承值。",
  "The concern is this lifecycle-dependent composable, not a blanket ban on ordinary utility functions or all Composition API calls outside setup. This contract has no current producer.":
    "问题针对这个依赖生命周期的组合式函数，并非全面禁止普通工具函数或 setup 外的所有 Composition API 调用。此诊断约定目前没有实现。",
  "The current producer scans template expressions such as JSON.parse(input). It does not report a throw statement that exists only in the script block.":
    "当前诊断实现扫描 JSON.parse(input) 等模板表达式，不会报告仅存在于 script 块中的 throw 语句。",
  "The example assumes IDs uniquely identify records. Comparing two references to the same reactive proxy remains valid; this contract has no current producer.":
    "此示例假定 ID 唯一标识记录。比较指向同一响应式代理的两个引用仍然有效；此诊断约定目前没有实现。",
  "The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.":
    "实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。",
  "The watcher must only derive the destination. Editable copies and callbacks with other side effects are allowed.":
    "此情形要求侦听器仅派生目标值。可编辑副本及具有其他副作用的回调被允许。",
  "This check compares explicit provider/consumer type annotations, not inferred literal value types. Keep the provider's `as string` annotation in this example.":
    "此检查比较提供者与消费者的显式类型注解，而非推断的字面量值类型。保留示例中提供者的 `as string` 注解。",
  "This example configures window.localStorage. The rule has no default deny list; enabling it alone does not report a member.":
    "此示例配置 window.localStorage。规则没有默认拒绝列表；仅启用规则不会报告成员。",
  "This example uses component provide/inject. `app.provide` and supported `app.runWithContext` injection are different valid ownership surfaces, not prohibited by this scenario. No current producer emits this contract code.":
    "此示例使用组件 provide/inject。`app.provide` 和受支持的 `app.runWithContext` 注入是不同且有效的所有权接口，不受此情形禁止。目前没有实现发出此诊断约定代码。",
  "This illustrates a concrete eager-initialization cycle. A recursive Vue component or every circular import is not automatically erroneous. No current producer emits this contract code.":
    "此示例展示具体的立即初始化循环。递归 Vue 组件或任意循环导入并不自动构成错误。目前没有实现发出此诊断约定代码。",
  "This illustrates the published preference for purely derived state. Watchers remain appropriate for external effects or independently writable state; no current producer emits this contract.":
    "此示例展示公开的纯派生状态偏好。侦听器仍适用于外部副作用或独立可写状态；目前没有实现发出此诊断约定。",
  "This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.":
    "这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。",
  "This is an explicit immutable-history ownership policy, not a general prohibition on passing or later mutating reactive objects. No current producer emits this contract.":
    "这是显式的不可变历史所有权策略，并非普遍禁止传递或随后修改响应式对象。目前没有实现发出此诊断约定。",
  "This is an explicitly chosen project layout policy; it does not invent a supported depth threshold or option. There is no current diagnostic producer for this contract.":
    "这是显式选择的项目布局策略，并未宣称存在受支持的深度阈值或选项。此诊断约定目前没有实现。",
  "This rule is a placeholder with an empty callback. Adding vapor selects Vapor compilation; the current linter does not report this catalog ID for its absence.":
    "此规则是具有空回调的占位项。添加 vapor 会选择 Vapor 编译；当前 linter 不会因缺少该属性而报告此目录 ID。",
  "Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.":
    "类型感知检查使用原生 Corsa 运行时及 TypeScript 项目。仅设置 `typeAware` 不会启用需显式选择的规则。",
  "Use a block-body arrow for the currently supported SFC filter. The underlying validator also handles method shorthand, but the current SFC prefilter does not reliably dispatch that shape.":
    "为当前支持的 SFC 筛选使用块函数体箭头函数。底层校验器也处理方法简写，但当前 SFC 前置筛选无法可靠分派该形式。",
  "Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.":
    "错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。",
  "Vue permits ref/reactive/computed outside component setup. The risk here is unwanted ownership/sharing under an explicit instance-isolation policy, not API illegality. This contract has no current producer.":
    "Vue 允许在组件 setup 外使用 ref/reactive/computed。此处风险是在显式实例隔离策略下产生非预期所有权与共享，并非 API 不合法。此诊断约定目前没有实现。",
};
