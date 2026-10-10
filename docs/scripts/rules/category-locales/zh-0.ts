export const chineseRules0: Record<string, readonly [string, string, string]> = {
  "a11y/alt-text": [
    "要求媒体元素提供替代文本",
    "图片提交控件只提供图片 URL，没有描述操作的 `alt` 文本。",
    '`alt="Submit search"` 为图片控件提供描述提交搜索操作的无障碍名称。',
  ],
  "a11y/anchor-has-content": [
    "要求锚点元素具有可访问的内容",
    "`/settings` 链接没有文本或其他命名内容，因此其目标没有可访问的描述。",
    "可见的 `Settings` 文本为同一目标链接提供内容。",
  ],
  "a11y/anchor-is-valid": [
    "要求锚点元素具有有效的 href",
    "第一个锚点使用 `#` 执行操作；第二个使用 JavaScript URL。两者都没有提供普通导航目标。",
    "原生按钮执行 `openPanel`，剩余锚点则具有真实的 `/docs/javascript-urls` 目标。",
  ],
  "a11y/aria-props": [
    "禁止无效的 ARIA 属性",
    "`aria-lable` 拼写错误，不是受支持的 ARIA 属性。",
    "受支持的 `aria-label` 属性提供按钮名称。",
  ],
  "a11y/aria-role": [
    "具有 ARIA 角色的元素必须使用有效的非抽象 ARIA 角色",
    "`datepicker` 不是此 section 上可识别的 ARIA 角色。",
    "section 使用可识别的 `dialog` 角色，以及描述日期选择的标签。",
  ],
  "a11y/aria-unsupported-elements": [
    "禁止在不支持 ARIA 的元素上使用 ARIA 属性",
    "元数据元素带有 `aria-hidden`，但 `meta` 不支持 ARIA 属性。",
    "移除 ARIA 属性，保留字符集声明。",
  ],
  "a11y/click-events-have-key-events": [
    "要求点击事件配有键盘事件处理器",
    "非交互式 `div` 有点击处理器，却没有键盘事件处理。",
    "原生 `button` 为同一 `activate` 处理器提供键盘激活方式。",
  ],
  "a11y/form-control-has-label": [
    "要求表单控件具有相关联的标签",
    "搜索输入框没有说明用户应输入什么的标签。",
    "将输入框包在 label 中，使可见的 `Search` 文本与控件关联。",
  ],
  "a11y/heading-has-content": [
    "要求标题元素具有可访问的内容",
    "`h2` 提供了标题层级，却没有标题内容。",
    "`Billing settings` 为原有二级标题提供内容。",
  ],
  "a11y/heading-levels": [
    "禁止跳过标题层级",
    "标题序列从 `h1` 直接跳到 `h3`，跳过二级。",
    "将账单标题改为 `h2`，保持连续的标题层级。",
  ],
  "a11y/iframe-has-title": [
    "要求 iframe 元素具有 title 属性",
    "结账框架有来源 URL，却没有描述嵌入内容的 `title`。",
    '`title="Checkout preview"` 为该框架的内容命名。',
  ],
  "a11y/img-alt": [
    "要求图片提供 alt 属性，以支持无障碍访问",
    "头像图片缺少 `alt` 属性。",
    '`alt="User avatar"` 为头像提供文本替代。',
  ],
  "a11y/interactive-supports-focus": [
    "要求具有交互角色的元素可获得焦点",
    "为 `span` 设置按钮角色和点击处理器，并不会让它能够通过键盘获得焦点。",
    "原生按钮可以获得焦点，并保留同一 `open` 操作。",
  ],
  "a11y/label-has-for": [
    "要求标签具有相关联的表单控件",
    "独立的 label 既没有通过 `for` 关联输入框，也没有包住输入框。",
    '`for="email"` 与输入框 ID 一致，显式关联两个元素。',
  ],
  "a11y/landmark-roles": [
    "验证地标角色的位置及唯一性",
    "同一模板中的两个 `main` 元素声明了重复的主地标。",
    "仪表盘保留为主地标；设置区域改为具名的导航地标。",
  ],
  "a11y/media-has-caption": [
    "要求媒体元素具有字幕",
    "视频有播放控件，却没有字幕轨道。",
    '带有 `kind="captions"` 的 `track` 为同一视频提供英文字幕。',
  ],
  "a11y/mouse-events-have-key-events": [
    "要求鼠标事件配有 focus/blur 事件",
    "预览可见性只通过鼠标进入和离开处理器改变。",
    "相同的预览操作也在 focus 和 blur 时执行，并且按钮可以获得键盘焦点。",
  ],
  "a11y/no-access-key": [
    "禁止使用 accesskey 属性",
    '`accesskey="s"` 快捷键可能与浏览器或辅助技术快捷键冲突。',
    "移除 `accesskey`，普通的 Save 按钮仍可使用。",
  ],
  "a11y/no-aria-hidden-on-focusable": [
    '禁止在可获得焦点的元素上使用 aria-hidden="true"',
    '可获得焦点的 Close 按钮使用 `aria-hidden="true"`，从无障碍树中隐藏。',
    "按钮继续对辅助技术可见，并获得 `Close` 标签，不再被隐藏。",
  ],
  "a11y/no-autofocus": [
    "禁止使用 autofocus 属性",
    "输入框在出现时请求自动获得焦点。",
    "移除 `autofocus`，避免该自动焦点请求，同时保留查询输入框。",
  ],
  "a11y/no-distracting-elements": [
    "禁止 &lt;marquee&gt; 和 &lt;blink&gt; 等分散注意力的元素",
    "`marquee` 元素引入了自动移动的文本。",
    "段落显示同样的优惠信息，不使用分散注意力的 marquee 元素。",
  ],
  "a11y/no-i-for-icon": [
    "禁止使用 &lt;i&gt; 元素表示图标",
    "图标通过 `i` 渲染，但其文本语义无法描述仅有图标的操作。",
    "装饰性 span 隐藏图标字形，独立的 `Delete item` 文本为按钮操作命名。",
  ],
  "a11y/no-redundant-roles": [
    "禁止多余的 ARIA 角色",
    '原生按钮已具有按钮角色，因此 `role="button"` 重复了其隐式语义。',
    "移除重复角色，保留 HTML 提供的按钮语义。",
  ],
  "a11y/no-refer-to-non-existent-id": [
    "禁止引用不存在的 ID",
    "`aria-labelledby` 指向 `save-label`，但没有元素声明该 ID。",
    "添加匹配的 span，解决引用问题并提供按钮标签。",
  ],
  "a11y/no-role-presentation-on-focusable": [
    '禁止在可获得焦点的元素上使用 role="presentation" 或 role="none"',
    "可获得焦点的账单链接请求 role=presentation，与其交互式链接角色冲突；浏览器必须忽略该呈现角色请求。",
    "移除冲突的呈现角色请求，使用原生链接角色和账单目标。",
  ],
  "a11y/no-static-element-interactions": [
    "禁止在静态元素上设置事件处理器",
    "静态 section 接收 Enter 键操作，却没有交互角色。",
    "原生按钮以适当的交互元素承载相同操作。",
  ],
  "a11y/placeholder-label-option": [
    "要求 select 的占位选项具有 disabled 或 hidden",
    "空值提示仍可被选择，仿佛它是一个国家值。",
    "添加 `disabled`，将提示与可选择的 Japan 选项区分开。",
  ],
  "a11y/role-has-required-aria-props": [
    "要求 ARIA 角色具有必需属性",
    "checkbox 角色省略了表达复选框状态的 `aria-checked`。",
    '`aria-checked="false"` 提供 checkbox 角色所需的状态。',
  ],
  "a11y/tabindex-no-positive": [
    "禁止正数 tabindex 值",
    "正数 tabindex 3 建立了位于普通控件之前的自定义焦点顺序。",
    "按钮使用原生焦点顺序，不设置正数 tabindex。",
  ],
  "a11y/use-list": [
    "建议为类似项目符号的文本使用列表元素",
    "任务项使用单独的段落和手写短横线标记，没有使用列表元素。",
    "无序列表及列表项以列表语义表达相同任务。",
  ],
  "css/no-display-none": [
    "建议使用 v-show 代替 display: none",
    "`.message` 声明通过 CSS 隐藏当前段落，没有使用模板可见性条件。",
    '`v-show="isSaved"` 在当前段落上显式表达可见性条件，并移除 `display: none`。',
  ],
  "css/no-hardcoded-values": [
    "建议使用 CSS 变量代替硬编码值",
    "按钮将间距数字和十六进制颜色直接写入声明。",
    "声明引用具名的间距和颜色自定义属性，使这些值可作为设计令牌维护。",
  ],
  "css/no-id-selectors": [
    "不建议在 CSS 中使用 ID 选择器",
    "`#submit` 将样式规则绑定到 ID 选择器。",
    "`.submit` 类提供可复用的样式入口，不使用 ID 选择器。",
  ],
  "css/no-important": [
    "不建议在 CSS 中使用 !important",
    "颜色声明通过 `!important` 覆盖普通层叠优先级。",
    "颜色来自自定义属性，没有使用 important 声明。",
  ],
  "css/no-utility-classes": [
    "警告在组件样式中实现工具类",
    "编写的选择器采用 `.flex`、`.mt-4` 和 `.text-center` 等工具类形式的名称。",
    "组件专用的 `.my-component` 选择器以一个语义名称组织组件样式。",
  ],
  "css/no-v-bind-performance": [
    "警告 CSS v-bind() 的性能开销",
    "样式表通过 SFC CSS 的 `v-bind()` 机制读取变化中的 `offset`。",
    "元素通过自身的样式绑定直接接收变化中的 transform。",
  ],
};
