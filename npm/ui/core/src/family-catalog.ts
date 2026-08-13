/** Lifecycle state exposed by the public family catalog. */
export type UiFamilyMaturity = "stable" | "preview" | "experimental" | "deprecated";

/** Runtime and build lanes that provide release evidence for one family. */
export type UiFamilyQualityGate =
  | "behavior-contract"
  | "mounted-dom"
  | "type-inference"
  | "ssr"
  | "hydration"
  | "dom-compile"
  | "ssr-compile"
  | "vapor-compile"
  | "tree-shaking"
  | "bundle-size";

/** Bundle limits proven through the package consumer tree-shaking gate. */
export interface UiFamilyBundleBudget {
  /** Export that keeps this family observable in a production consumer. */
  readonly exportName: string;

  /** Regular expression source expected in this family's bundled output. */
  readonly retainedSignature: string;

  /** Other families intentionally retained because they are runtime dependencies. */
  readonly allowedRetainedFamilies?: readonly string[];

  /** Maximum minified JavaScript gzip bytes for the root and subpath consumer. */
  readonly maximumJavaScriptGzipBytes: number;

  /** Maximum extracted CSS gzip bytes for the root and subpath consumer. */
  readonly maximumCssGzipBytes: number;
}

/** One source-owned UI component or foundation family. */
export interface UiFamilyCatalogEntry {
  /** Stable machine-readable name used by subpaths, tests, and issue ledgers. */
  readonly canonicalName: string;

  /** Human-readable family name for generated catalogs and docs. */
  readonly title: string;

  /** Public package subpath that owns this family. */
  readonly packageSubpath: "." | `./${string}`;

  /** Source entry compiled into the public package subpath. */
  readonly entryFile: `src/${string}.ts`;

  /** Canonical source files that define the family contract. */
  readonly sourceFiles: readonly `src/${string}`[];

  /** Normative behavior table for this family. */
  readonly behaviorContract: `src/${string}.behavior.md`;

  /** Runtime or mounted-DOM tests that exercise behavior. */
  readonly tests: readonly `src/${string}.test.ts`[];

  /** Compile-only public type tests. */
  readonly typeTests?: readonly `src/${string}.types.test-d.ts`[];

  /** Renderer fixture file checked by scripts/check-renderers.ts, when applicable. */
  readonly rendererFixture?: `${string}Consumer.vue` | `${string}.vue`;

  /** Enforced quality gates that must have concrete artifacts. */
  readonly qualityGates: readonly UiFamilyQualityGate[];

  /** Bundle budget and unused-family elimination contract. */
  readonly bundleBudget?: UiFamilyBundleBudget;

  /** Alternate names recognized for discovery and migration. */
  readonly aliases: readonly string[];

  /** Upstream families or primitives this entry covers semantically. */
  readonly upstreamCoverage: readonly string[];

  /** Other catalogued families required by this implementation. */
  readonly dependencies: readonly string[];

  /** Release lifecycle state. Stable entries must pass every declared gate. */
  readonly maturity: UiFamilyMaturity;

  /** Owning area responsible for keeping the catalog entry current. */
  readonly owner: string;
}

export const UI_FAMILY_CATALOG_SCHEMA_VERSION = 1;

const stableQualityGates = [
  "behavior-contract",
  "mounted-dom",
  "type-inference",
  "tree-shaking",
  "bundle-size",
] as const;

const rendererQualityGates = ["dom-compile", "ssr-compile", "vapor-compile"] as const;

const interactionQualityGates = [
  ...stableQualityGates,
  "ssr",
  "hydration",
  ...rendererQualityGates,
] as const;

const componentQualityGates = [
  ...stableQualityGates,
  "ssr",
  "hydration",
  ...rendererQualityGates,
] as const;

const owner = "ui-foundations";

export const uiFamilyCatalog = [
  {
    canonicalName: "button",
    title: "Button",
    packageSubpath: "./button",
    entryFile: "src/button.ts",
    sourceFiles: ["src/ActionButton.vue", "src/button.ts", "src/button-keyboard.ts"],
    behaviorContract: "src/button.behavior.md",
    tests: ["src/button.test.ts"],
    rendererFixture: "ActionButton.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "Button",
      retainedSignature: "aria-busy",
      maximumJavaScriptGzipBytes: 1_000,
      maximumCssGzipBytes: 0,
    },
    aliases: ["action-button", "button primitive"],
    upstreamCoverage: ["shadcn/ui Button", "Reka UI Primitive", "React Aria Button"],
    dependencies: ["primitive", "press"],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "checkbox",
    title: "Checkbox",
    packageSubpath: "./checkbox",
    entryFile: "src/checkbox.ts",
    sourceFiles: ["src/CheckboxControl.vue", "src/checkbox.ts", "src/checkbox-state.ts"],
    behaviorContract: "src/checkbox.behavior.md",
    tests: ["src/checkbox.test.ts"],
    rendererFixture: "CheckboxControl.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "Checkbox",
      retainedSignature: "aria-checked",
      allowedRetainedFamilies: ["controllable-state"],
      maximumJavaScriptGzipBytes: 1_100,
      maximumCssGzipBytes: 0,
    },
    aliases: ["checkbox control", "mixed checkbox"],
    upstreamCoverage: ["shadcn/ui Checkbox", "Reka UI Checkbox", "React Aria Checkbox"],
    dependencies: ["controllable-state"],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "collection",
    title: "Collection Registry",
    packageSubpath: "./collection",
    entryFile: "src/collection.ts",
    sourceFiles: [
      "src/collection.ts",
      "src/collection-registry.ts",
      "src/collection-keys.ts",
      "src/collection-observer.ts",
      "src/collection-order.ts",
      "src/collection-text.ts",
      "src/collection-types.ts",
    ],
    behaviorContract: "src/collection.behavior.md",
    tests: ["src/collection.test.ts", "src/collection-dom.test.ts"],
    typeTests: ["src/collection.types.test-d.ts"],
    qualityGates: stableQualityGates,
    bundleBudget: {
      exportName: "createCollectionRegistry",
      retainedSignature: "VIZE_UI_COLLECTION_DISPOSED",
      maximumJavaScriptGzipBytes: 3_150,
      maximumCssGzipBytes: 0,
    },
    aliases: ["collection registry", "item registry", "logical focus registry"],
    upstreamCoverage: ["React Aria Collection", "Ariakit Composite Store", "Zag collection"],
    dependencies: [],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "composite-navigation",
    title: "Composite Navigation",
    packageSubpath: "./composite-navigation",
    entryFile: "src/composite-navigation.ts",
    sourceFiles: [
      "src/composite-navigation.ts",
      "src/composite-navigation-dom.ts",
      "src/composite-navigation-internal.ts",
      "src/composite-navigation-types.ts",
    ],
    behaviorContract: "src/composite-navigation.behavior.md",
    tests: [
      "src/composite-navigation.test.ts",
      "src/composite-navigation-active-descendant.test.ts",
      "src/composite-navigation-lifecycle.test.ts",
      "src/composite-navigation-ssr.test.ts",
    ],
    typeTests: ["src/composite-navigation.types.test-d.ts"],
    rendererFixture: "CompositeNavigationConsumer.vue",
    qualityGates: interactionQualityGates,
    bundleBudget: {
      exportName: "createCompositeNavigation",
      retainedSignature: "VIZE_UI_COMPOSITE_NAVIGATION_DISPOSED",
      allowedRetainedFamilies: ["typeahead"],
      maximumJavaScriptGzipBytes: 3_900,
      maximumCssGzipBytes: 0,
    },
    aliases: ["roving focus", "active descendant", "composite widget navigation"],
    upstreamCoverage: ["WAI-ARIA composite widgets", "Reka UI Roving Focus", "Ariakit Composite"],
    dependencies: ["collection", "typeahead"],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "context",
    title: "Typed Component Context",
    packageSubpath: "./context",
    entryFile: "src/context.ts",
    sourceFiles: ["src/context.ts"],
    behaviorContract: "src/context.behavior.md",
    tests: ["src/context.test.ts"],
    typeTests: ["src/context.types.test-d.ts"],
    qualityGates: stableQualityGates,
    bundleBudget: {
      exportName: "createContext",
      retainedSignature: "VIZE_UI_CONTEXT_MISSING",
      maximumJavaScriptGzipBytes: 450,
      maximumCssGzipBytes: 0,
    },
    aliases: ["provide inject context", "compound component context"],
    upstreamCoverage: ["Reka UI createContext", "Radix context scope"],
    dependencies: [],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "controllable-state",
    title: "Controlled and Uncontrolled State",
    packageSubpath: "./controllable-state",
    entryFile: "src/controllable-state.ts",
    sourceFiles: ["src/controllable-state.ts"],
    behaviorContract: "src/controllable-state.behavior.md",
    tests: ["src/controllable-state.test.ts"],
    typeTests: ["src/controllable-state.types.test-d.ts"],
    qualityGates: stableQualityGates,
    bundleBudget: {
      exportName: "useControllableState",
      retainedSignature: "defaultValue[\\s\\S]*Object\\.is[\\s\\S]*defaultValue",
      maximumJavaScriptGzipBytes: 550,
      maximumCssGzipBytes: 0,
    },
    aliases: ["controllable state", "controlled state", "uncontrolled state"],
    upstreamCoverage: ["Radix useControllableState", "React Aria controlled state"],
    dependencies: [],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "focus",
    title: "Focus Interactions",
    packageSubpath: "./focus",
    entryFile: "src/focus.ts",
    sourceFiles: ["src/focus.ts", "src/focus-internal.ts", "src/focus-types.ts"],
    behaviorContract: "src/focus.behavior.md",
    tests: ["src/focus.test.ts", "src/focus-lifecycle.test.ts", "src/focus-ssr.test.ts"],
    typeTests: ["src/focus.types.test-d.ts"],
    rendererFixture: "FocusConsumer.vue",
    qualityGates: interactionQualityGates,
    bundleBudget: {
      exportName: "createFocus",
      retainedSignature: "VIZE_UI_FOCUS_DISPOSED",
      allowedRetainedFamilies: ["interaction-modality"],
      maximumJavaScriptGzipBytes: 3_300,
      maximumCssGzipBytes: 0,
    },
    aliases: ["focus ring", "focus within", "focus visible"],
    upstreamCoverage: ["React Aria useFocus", "React Aria useFocusRing", "WICG focus-visible"],
    dependencies: ["interaction-modality"],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "focus-guards",
    title: "Focus Guards",
    packageSubpath: "./focus-guards",
    entryFile: "src/focus-guards.ts",
    sourceFiles: [
      "src/focus-guards.ts",
      "src/focus-guards-internal.ts",
      "src/focus-guards-stack.ts",
      "src/focus-guards-types.ts",
    ],
    behaviorContract: "src/focus-guards.behavior.md",
    tests: ["src/focus-guards.test.ts", "src/focus-guards-ssr.test.ts"],
    typeTests: ["src/focus-guards.types.test-d.ts"],
    rendererFixture: "FocusGuardsConsumer.vue",
    qualityGates: interactionQualityGates,
    bundleBudget: {
      exportName: "createFocusGuards",
      retainedSignature: "VIZE_UI_FOCUS_GUARDS_DISPOSED",
      maximumJavaScriptGzipBytes: 3_500,
      maximumCssGzipBytes: 0,
    },
    aliases: ["focus sentinels", "tab guards", "focus trap guards"],
    upstreamCoverage: ["Radix FocusGuards", "React Aria FocusScope sentinels"],
    dependencies: ["focus-scope"],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "focus-scope",
    title: "Focus Scope",
    packageSubpath: "./focus-scope",
    entryFile: "src/focus-scope.ts",
    sourceFiles: [
      "src/focus-scope.ts",
      "src/focus-scope-dom.ts",
      "src/focus-scope-internal.ts",
      "src/focus-scope-manager.ts",
      "src/focus-scope-stack.ts",
      "src/focus-scope-types.ts",
      "src/use-focus-scope.ts",
    ],
    behaviorContract: "src/focus-scope.behavior.md",
    tests: [
      "src/focus-scope.test.ts",
      "src/focus-scope-lifecycle.test.ts",
      "src/focus-scope-ssr.test.ts",
    ],
    typeTests: ["src/focus-scope.types.test-d.ts"],
    rendererFixture: "FocusScopeConsumer.vue",
    qualityGates: interactionQualityGates,
    bundleBudget: {
      exportName: "createFocusScope",
      retainedSignature: "VIZE_UI_FOCUS_SCOPE_DISPOSED",
      maximumJavaScriptGzipBytes: 4_050,
      maximumCssGzipBytes: 0,
    },
    aliases: ["focus trap", "focus containment", "focus restoration"],
    upstreamCoverage: ["React Aria FocusScope", "Radix FocusScope", "Ariakit FocusTrap"],
    dependencies: [],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "hover",
    title: "Hover Interaction",
    packageSubpath: "./hover",
    entryFile: "src/hover.ts",
    sourceFiles: ["src/hover.ts", "src/hover-types.ts"],
    behaviorContract: "src/hover.behavior.md",
    tests: ["src/hover.test.ts", "src/hover-ssr.test.ts"],
    typeTests: ["src/hover.types.test-d.ts"],
    rendererFixture: "HoverConsumer.vue",
    qualityGates: interactionQualityGates,
    bundleBudget: {
      exportName: "createHover",
      retainedSignature: "VIZE_UI_HOVER_DISPOSED",
      maximumJavaScriptGzipBytes: 1_450,
      maximumCssGzipBytes: 0,
    },
    aliases: ["hover", "pointer hover", "pen hover"],
    upstreamCoverage: ["React Aria useHover", "Pointer Events hover"],
    dependencies: [],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "id",
    title: "Deterministic ID Provider",
    packageSubpath: "./id",
    entryFile: "src/id.ts",
    sourceFiles: ["src/DeterministicIdProvider.vue", "src/id.ts", "src/deterministic-id.ts"],
    behaviorContract: "src/id.behavior.md",
    tests: ["src/id.test.ts"],
    typeTests: ["src/id.types.test-d.ts"],
    rendererFixture: "DeterministicIdProvider.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "IdProvider",
      retainedSignature: "DeterministicIdProvider",
      maximumJavaScriptGzipBytes: 1_050,
      maximumCssGzipBytes: 0,
    },
    aliases: ["id provider", "deterministic ids", "hydration ids"],
    upstreamCoverage: ["React Aria SSRProvider", "Vue useId"],
    dependencies: ["context"],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "inert-outside",
    title: "Outside Inerting",
    packageSubpath: "./inert-outside",
    entryFile: "src/inert-outside.ts",
    sourceFiles: [
      "src/inert-outside.ts",
      "src/inert-outside-dom.ts",
      "src/inert-outside-internal.ts",
      "src/inert-outside-stack.ts",
      "src/inert-outside-types.ts",
    ],
    behaviorContract: "src/inert-outside.behavior.md",
    tests: ["src/inert-outside.test.ts", "src/inert-outside-ssr.test.ts"],
    typeTests: ["src/inert-outside.types.test-d.ts"],
    rendererFixture: "InertOutsideConsumer.vue",
    qualityGates: interactionQualityGates,
    bundleBudget: {
      exportName: "createInertOutside",
      retainedSignature: "VIZE_UI_INERT_OUTSIDE_DISPOSED",
      maximumJavaScriptGzipBytes: 2_175,
      maximumCssGzipBytes: 0,
    },
    aliases: ["aria hidden outside", "outside inerting", "overlay inerting"],
    upstreamCoverage: ["Radix overlay inerting", "Adobe Spectrum overlay provider"],
    dependencies: [],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "interaction-modality",
    title: "Interaction Modality",
    packageSubpath: "./interaction-modality",
    entryFile: "src/interaction-modality.ts",
    sourceFiles: [
      "src/interaction-modality.ts",
      "src/interaction-modality-events.ts",
      "src/interaction-modality-hub.ts",
      "src/interaction-modality-types.ts",
    ],
    behaviorContract: "src/interaction-modality.behavior.md",
    tests: [
      "src/interaction-modality.test.ts",
      "src/interaction-modality-lifecycle.test.ts",
      "src/interaction-modality-ssr.test.ts",
    ],
    typeTests: ["src/interaction-modality.types.test-d.ts"],
    qualityGates: [
      "behavior-contract",
      "mounted-dom",
      "type-inference",
      "ssr",
      "tree-shaking",
      "bundle-size",
    ],
    bundleBudget: {
      exportName: "createInteractionModalityTracker",
      retainedSignature: "VIZE_UI_INTERACTION_MODALITY_DISPOSED",
      maximumJavaScriptGzipBytes: 1_650,
      maximumCssGzipBytes: 0,
    },
    aliases: ["focus modality", "input modality", "focus visible modality"],
    upstreamCoverage: ["React Aria useFocusVisible", "WICG focus-visible"],
    dependencies: [],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "long-press",
    title: "Long Press Interaction",
    packageSubpath: "./long-press",
    entryFile: "src/long-press.ts",
    sourceFiles: ["src/long-press.ts", "src/long-press-internal.ts", "src/long-press-types.ts"],
    behaviorContract: "src/long-press.behavior.md",
    tests: [
      "src/long-press.test.ts",
      "src/long-press-hardening.test.ts",
      "src/long-press-legacy.test.ts",
      "src/long-press-ssr.test.ts",
    ],
    typeTests: ["src/long-press.types.test-d.ts"],
    rendererFixture: "LongPressConsumer.vue",
    qualityGates: interactionQualityGates,
    bundleBudget: {
      exportName: "createLongPress",
      retainedSignature: "VIZE_UI_LONG_PRESS_DISPOSED",
      allowedRetainedFamilies: ["press"],
      maximumJavaScriptGzipBytes: 5_150,
      maximumCssGzipBytes: 0,
    },
    aliases: ["long press", "press and hold"],
    upstreamCoverage: ["React Aria useLongPress"],
    dependencies: ["press"],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "move",
    title: "Move Interaction",
    packageSubpath: "./move",
    entryFile: "src/move.ts",
    sourceFiles: ["src/move.ts", "src/move-internal.ts", "src/move-types.ts"],
    behaviorContract: "src/move.behavior.md",
    tests: ["src/move.test.ts", "src/move-internal.test.ts", "src/move-ssr.test.ts"],
    typeTests: ["src/move.types.test-d.ts"],
    rendererFixture: "MoveConsumer.vue",
    qualityGates: interactionQualityGates,
    bundleBudget: {
      exportName: "createMove",
      retainedSignature: "VIZE_UI_MOVE_DISPOSED",
      maximumJavaScriptGzipBytes: 2_750,
      maximumCssGzipBytes: 0,
    },
    aliases: ["move", "drag movement", "keyboard move"],
    upstreamCoverage: ["React Aria useMove", "Pointer Events pointer capture"],
    dependencies: [],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "press",
    title: "Press Interaction",
    packageSubpath: "./press",
    entryFile: "src/press.ts",
    sourceFiles: [
      "src/press.ts",
      "src/press-activation-memory.ts",
      "src/press-event.ts",
      "src/press-handlers.ts",
      "src/press-lifecycle.ts",
      "src/press-types.ts",
    ],
    behaviorContract: "src/press.behavior.md",
    tests: [
      "src/press.test.ts",
      "src/press-lifecycle.test.ts",
      "src/press-legacy.test.ts",
      "src/press-ssr.test.ts",
    ],
    typeTests: ["src/press.types.test-d.ts"],
    rendererFixture: "PressConsumer.vue",
    qualityGates: interactionQualityGates,
    bundleBudget: {
      exportName: "createPress",
      retainedSignature: "VIZE_UI_PRESS_DISPOSED",
      maximumJavaScriptGzipBytes: 3_550,
      maximumCssGzipBytes: 0,
    },
    aliases: ["press", "activation", "pointer press"],
    upstreamCoverage: ["React Aria usePress", "Pointer Events activation"],
    dependencies: [],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "primitive",
    title: "Polymorphic Primitive",
    packageSubpath: "./primitive",
    entryFile: "src/primitive.ts",
    sourceFiles: ["src/PrimitiveElement.vue", "src/primitive.ts"],
    behaviorContract: "src/primitive.behavior.md",
    tests: ["src/primitive.test.ts"],
    rendererFixture: "PrimitiveElement.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "Primitive",
      retainedSignature: "data-vize-ui.+primitive",
      maximumJavaScriptGzipBytes: 500,
      maximumCssGzipBytes: 0,
    },
    aliases: ["as child", "polymorphic primitive", "slot forwarding primitive"],
    upstreamCoverage: ["Reka UI Primitive", "Radix Slot", "shadcn/ui Slot"],
    dependencies: [],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "scroll-lock",
    title: "Scroll Lock",
    packageSubpath: "./scroll-lock",
    entryFile: "src/scroll-lock.ts",
    sourceFiles: [
      "src/scroll-lock.ts",
      "src/scroll-lock-dom.ts",
      "src/scroll-lock-internal.ts",
      "src/scroll-lock-stack.ts",
      "src/scroll-lock-types.ts",
    ],
    behaviorContract: "src/scroll-lock.behavior.md",
    tests: ["src/scroll-lock.test.ts", "src/scroll-lock-ssr.test.ts"],
    typeTests: ["src/scroll-lock.types.test-d.ts"],
    rendererFixture: "ScrollLockConsumer.vue",
    qualityGates: interactionQualityGates,
    bundleBudget: {
      exportName: "createScrollLock",
      retainedSignature: "VIZE_UI_SCROLL_LOCK_DISPOSED",
      maximumJavaScriptGzipBytes: 2_150,
      maximumCssGzipBytes: 0,
    },
    aliases: ["body scroll lock", "document scroll lock", "overlay scroll lock"],
    upstreamCoverage: ["React Aria usePreventScroll", "Radix RemoveScroll"],
    dependencies: [],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "spatial-navigation",
    title: "Spatial Navigation",
    packageSubpath: "./spatial-navigation",
    entryFile: "src/spatial-navigation.ts",
    sourceFiles: [
      "src/spatial-navigation.ts",
      "src/spatial-navigation-internal.ts",
      "src/spatial-navigation-types.ts",
    ],
    behaviorContract: "src/spatial-navigation.behavior.md",
    tests: [
      "src/spatial-navigation.test.ts",
      "src/spatial-navigation-lifecycle.test.ts",
      "src/spatial-navigation-ssr.test.ts",
    ],
    typeTests: ["src/spatial-navigation.types.test-d.ts"],
    rendererFixture: "SpatialNavigationConsumer.vue",
    qualityGates: interactionQualityGates,
    bundleBudget: {
      exportName: "createSpatialNavigation",
      retainedSignature: "VIZE_UI_SPATIAL_NAVIGATION_DISPOSED",
      maximumJavaScriptGzipBytes: 2_600,
      maximumCssGzipBytes: 0,
    },
    aliases: ["spatial navigation", "grid navigation", "geometry navigation"],
    upstreamCoverage: ["CSS Spatial Navigation", "ARIA grid navigation"],
    dependencies: ["collection"],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "typeahead",
    title: "Collection Typeahead",
    packageSubpath: "./typeahead",
    entryFile: "src/typeahead.ts",
    sourceFiles: ["src/typeahead.ts", "src/typeahead-internal.ts", "src/typeahead-types.ts"],
    behaviorContract: "src/typeahead.behavior.md",
    tests: ["src/typeahead.test.ts", "src/typeahead-ssr.test.ts"],
    typeTests: ["src/typeahead.types.test-d.ts"],
    rendererFixture: "TypeaheadConsumer.vue",
    qualityGates: interactionQualityGates,
    bundleBudget: {
      exportName: "createTypeahead",
      retainedSignature: "VIZE_UI_TYPEAHEAD_DISPOSED",
      maximumJavaScriptGzipBytes: 1_375,
      maximumCssGzipBytes: 0,
    },
    aliases: ["typeahead", "collection search", "keyboard search"],
    upstreamCoverage: ["React Aria useTypeSelect", "Ariakit composite typeahead"],
    dependencies: ["collection"],
    maturity: "stable",
    owner,
  },
  {
    canonicalName: "visually-hidden",
    title: "Visually Hidden",
    packageSubpath: "./visually-hidden",
    entryFile: "src/visually-hidden.ts",
    sourceFiles: ["src/VisuallyHidden.vue", "src/visually-hidden.ts"],
    behaviorContract: "src/visually-hidden.behavior.md",
    tests: ["src/visually-hidden.test.ts"],
    rendererFixture: "VisuallyHidden.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "VisuallyHidden",
      retainedSignature: "visually-hidden",
      maximumJavaScriptGzipBytes: 400,
      maximumCssGzipBytes: 180,
    },
    aliases: ["screen reader only", "sr-only", "visually hidden"],
    upstreamCoverage: ["React Aria VisuallyHidden", "Radix VisuallyHidden"],
    dependencies: [],
    maturity: "stable",
    owner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];

export type UiFamilyCatalog = typeof uiFamilyCatalog;
export type UiFamilyName = UiFamilyCatalog[number]["canonicalName"];
