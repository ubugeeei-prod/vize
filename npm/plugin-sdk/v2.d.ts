import type { FactGroups } from "./facts.js";
import type { PluginNode, ScopeEntry } from "./index.js";

/** Experimental integrated plugin API generation. */
export declare const PLUGIN_API_VERSION: 2;
/** Registered symbol that hosts read from every `vue.plugins` entry. */
export declare const PLUGIN_ENTRY: unique symbol;

/** Canonical plugin data: JSON without non-integer numbers. */
export type PluginData =
  | null
  | boolean
  | number
  | string
  | readonly PluginData[]
  | { readonly [key: string]: PluginData | undefined };

/** Configured severity of a plugin rule. */
export type PluginLintLevel = "off" | "warn" | "error";
/** Whether a rule's fixes are safe to apply automatically. */
export type PluginLintFixable = "none" | "safe" | "unsafe";
/** Public document domains a schema 1 rule may inspect. */
export type PluginLintDocument = "template";
/** Public template record kinds a rule may visit. */
export type PluginRecordKind =
  | "ui.element"
  | "ui.component"
  | "ui.text"
  | "ui.interpolation"
  | "ui.comment"
  | "ui.if"
  | "ui.for"
  | "ui.slot"
  | "ui.bind"
  | "ui.on"
  | "ui.model"
  | "ui.slot-content"
  | "vue.directive"
  | "vue.css-bind"
  | "vue.sync"
  | "vue.slot-scope"
  | "vue.once"
  | "vue.memo"
  | "vue.show"
  | "vue.html"
  | "vue.text"
  | "vue.cloak";
/** Native fact groups a rule may demand. */
export type PluginFactGroup = keyof FactGroups | "templateScopes";

/** One report from a v2 rule; the host owns ranges, severity and rendering. */
export interface PluginLintReport<MessageId extends string = string> {
  node: PluginNode;
  messageId: MessageId;
  /** Interpolation values for `{name}` placeholders. Defaults to none. */
  data?: Readonly<Record<string, string | number | boolean>>;
  /** Replacement text for the reported node's authored span. Defaults to no fix. */
  fix?: string;
}

/** Read-only records and reporting for one rule over one admitted batch. */
export interface PluginLintContext<MessageId extends string = string> {
  readonly file: string;
  readonly nodes: readonly PluginNode[];
  readonly options: PluginData;
  facts(name: "templateScopes"): ReadonlyMap<number, readonly ScopeEntry[]>;
  facts<K extends keyof FactGroups>(name: K): FactGroups[K];
  ancestors(node: PluginNode, kind: PluginRecordKind): IterableIterator<PluginNode>;
  report(report: PluginLintReport<MessageId>): void;
}

/** One plugin-namespaced lint rule over public template records. */
export interface PluginLintRule<MessageId extends string = string> {
  /** Kebab-case; the public rule ID is `<plugin name>/<id>`. */
  id: string;
  meta: {
    /** Rule documentation. Defaults to `""`. */
    docs?: string;
    /** Severity when the config does not name the rule. Defaults to `"off"`. */
    defaultLevel?: PluginLintLevel;
    /** Defaults to `"none"`. */
    fixable?: PluginLintFixable;
    messages: Readonly<Record<MessageId, string>>;
  };
  /** Defaults to `["template"]`. */
  documents?: readonly PluginLintDocument[];
  visit: readonly PluginRecordKind[];
  /** Static fact demands. Defaults to `[]`. */
  demands?: readonly PluginFactGroup[];
  run(context: PluginLintContext<MessageId>): void;
}

/** Contract of one boolean directive modifier. */
export interface PluginDirectiveModifier {
  /** Defaults to `true`. A required modifier cannot declare a default. */
  optional?: boolean;
  /** Defaults to `false`. */
  default?: boolean;
}

/** Contribution recorder passed to `setup`. */
export interface PluginSetupApi {
  readonly lint: {
    rule<MessageId extends string>(rule: PluginLintRule<MessageId>): void;
  };
  readonly sfc: {
    /**
     * Declared only: no product admits custom-block ownership yet.
     * `lang` defaults to every language of the block.
     */
    block(type: string, contract?: { lang?: string }): void;
  };
  readonly vue: {
    /**
     * Lint, the language server and `vize check` check modifier contracts, and
     * the compiler emits their defaults in VDOM, SSR and Vapor output.
     * Modifiers default to none.
     */
    directive(
      name: string,
      contract?: { modifiers?: Readonly<Record<string, PluginDirectiveModifier>> },
    ): void;
  };
}

/** A declared, inspectable permission. JavaScript callbacks remain trusted code. */
export type PluginPermission =
  | { kind: "env"; keys: readonly string[] }
  | { kind: "fs.read"; paths: readonly string[] };

/** Authored definition of an experimental v2 plugin. */
export interface VizePluginDefinition<Options extends PluginData> {
  apiVersion: 2;
  stability: "experimental";
  /** Package-style name; it namespaces every contribution. */
  name: string;
  version: string;
  /** Every field defaults to every supported value. */
  supports?: {
    vue?: readonly ("2" | "3")[];
    outputs?: readonly ("vdom" | "vapor" | "ssr")[];
    runtimes?: readonly ("node" | "bun")[];
  };
  /** Defaults to `[]`. */
  permissions?: readonly PluginPermission[];
  /** Every configuration or ambient value the callbacks read outside their batch. Defaults to `[]`. */
  cacheInputs?: readonly { name: string; value: string }[];
  /** Plugin names this plugin must run before. Defaults to `[]`. */
  before?: readonly string[];
  /** Plugin names this plugin must run after. Defaults to `[]`. */
  after?: readonly string[];
  setup(api: PluginSetupApi, options: Options): void;
}

/** A resolved `vue.plugins` entry. Its manifest is lowered data only. */
export interface VizePluginEntry {
  readonly name: string;
  readonly version: string;
  readonly [PLUGIN_ENTRY]: {
    readonly apiVersion: 2;
    readonly manifest: Readonly<Record<string, PluginData>>;
    readonly callbacks: readonly Readonly<Record<string, (...args: never[]) => unknown>>[];
  };
}

/** Define an experimental v2 plugin factory; its options default to `{}`. */
export declare function defineVizePlugin<Options extends PluginData = Record<string, never>>(
  definition: VizePluginDefinition<Options>,
): (options?: Options) => VizePluginEntry;
