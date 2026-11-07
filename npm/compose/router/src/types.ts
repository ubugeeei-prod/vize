/** A source-owned Vue route component. */
export type RouteComponentSpecifier = `${string}.vue`;

/** Serializable metadata attached to a route record. */
export type RouteMeta = Readonly<Record<string, unknown>>;

/** Query values preserve repeated keys without depending on URLSearchParams at hydration time. */
export type RouteQuery = Readonly<Record<string, string | readonly string[]>>;

/** Context passed to a route loader. */
export interface RouteLoaderContext<Params = RouteParams> {
  /** URL pathname that matched the route. */
  readonly pathname: string;

  /** Params decoded from dynamic path segments. */
  readonly params: Params;

  /** Parsed query string. Repeated keys become arrays in insertion order. */
  readonly query: RouteQuery;

  /** Hash without the leading `#`. */
  readonly hash: string;

  /** Abort signal owned by the current navigation or server render. */
  readonly signal?: AbortSignal;
}

/** Loader invoked by integrations after a route has matched. */
export type RouteLoader<Params, Data> = (
  context: RouteLoaderContext<Params>,
) => Data | Promise<Data>;

/** Typed route record accepted by defineRoutes. */
export interface RouteDefinition<
  Name extends string = string,
  Path extends string = string,
  Meta extends RouteMeta = RouteMeta,
  Data = unknown,
> {
  /** Literal route name used for typed navigation. */
  readonly name: Name;

  /** Absolute path pattern. Dynamic segments use `:id`; optional segments use `:tab?`. */
  readonly path: Path;

  /** `.vue` component source for the route. */
  readonly component: RouteComponentSpecifier;

  /** Serializable route metadata available on server and client. */
  readonly meta?: Meta;

  /** Optional data loader. Its param type is inferred from `path`. */
  readonly loader?: RouteLoader<Simplify<RoutePathParams<Path>>, Data>;

  /** Optional redirect evaluated before the loader. */
  readonly redirect?: RouteRedirect<Simplify<RoutePathParams<Path>>>;

  /** Source-owned error boundary selected when the loader or a guard fails. */
  readonly errorComponent?: RouteComponentSpecifier;
}

/** Redirect returned by a route record or navigation guard. */
export type RouteRedirect<Params = RouteParams> =
  | string
  | URL
  | ((context: RouteLoaderContext<Params>) => string | URL | Promise<string | URL>);

/** Matched route result shared by server and client runtimes. */
export interface RouteMatch<Route extends RouteDefinition = RouteDefinition> {
  readonly route: Route;
  readonly name: Route["name"];
  readonly pathname: string;
  readonly params: Simplify<RoutePathParams<Route["path"]>>;
  readonly query: RouteQuery;
  readonly hash: string;
  readonly meta: Route["meta"] extends RouteMeta ? Route["meta"] : RouteMeta;
}

/** Serialized route state embedded into SSR HTML and restored during hydration. */
export interface SerializedRouteState<Name extends string = string> {
  readonly schemaVersion: 1;
  readonly name: Name;
  readonly pathname: string;
  readonly params: RouteParams;
  readonly query: RouteQuery;
  readonly hash: string;
  readonly data?: unknown;
}

/** Options for typed URL generation. */
export interface RouteResolveOptions {
  /**
   * Query values appended to the generated URL.
   *
   * @default {}
   */
  readonly query?: Readonly<
    Record<
      string,
      string | number | boolean | readonly (string | number | boolean)[] | null | undefined
    >
  >;

  /**
   * Hash appended without requiring a leading `#`.
   *
   * @default ""
   */
  readonly hash?: string;
}

/** Machine-readable route manifest for generators and docs. */
export interface RouteManifest {
  readonly schemaVersion: 1;
  readonly routes: readonly RouteManifestEntry[];
}

/** Serializable manifest row for one route. */
export interface RouteManifestEntry {
  readonly name: string;
  readonly path: string;
  readonly component: RouteComponentSpecifier;
  readonly meta: RouteMeta;
  readonly errorComponent?: RouteComponentSpecifier;
}

/** A named, type-safe navigation target. */
export type NamedRouteLocation<
  Routes extends readonly RouteDefinition[],
  Name extends RouteNames<Routes> = RouteNames<Routes>,
> =
  Name extends RouteNames<Routes>
    ? {
        readonly name: Name;
        readonly params: RouteParamsForName<Routes, Name>;
        readonly query?: RouteResolveOptions["query"];
        readonly hash?: string;
      }
    : never;

/** Input accepted by the navigation lifecycle. */
export type RouteLocation<Routes extends readonly RouteDefinition[]> =
  | string
  | URL
  | NamedRouteLocation<Routes>;

export type NavigationHistoryMode = "push" | "replace" | "pop" | "none";
export type NavigationStatus = "idle" | "pending" | "success" | "cancelled" | "stale" | "error";

export interface NavigationOptions {
  /** History behavior after a successful navigation. @default "push" */
  readonly history?: NavigationHistoryMode;
  /** Scroll snapshot restored for a history traversal. */
  readonly scroll?: unknown;
  /** Focus snapshot restored for a history traversal. */
  readonly focus?: unknown;
}

export interface NavigationContext<Routes extends readonly RouteDefinition[]> {
  readonly from: RouteMatch<Routes[number]> | undefined;
  readonly to: RouteMatch<Routes[number]>;
  readonly signal: AbortSignal;
}

export type NavigationGuardResult<Routes extends readonly RouteDefinition[]> =
  | void
  | true
  | false
  | RouteLocation<Routes>;

export type NavigationGuard<Routes extends readonly RouteDefinition[]> = (
  context: NavigationContext<Routes>,
) => NavigationGuardResult<Routes> | Promise<NavigationGuardResult<Routes>>;

export type NavigationResult<Routes extends readonly RouteDefinition[]> =
  | {
      readonly status: "success";
      readonly match: RouteMatch<Routes[number]>;
      readonly data: unknown;
      readonly redirected: boolean;
    }
  | { readonly status: "cancelled" | "stale"; readonly match?: RouteMatch<Routes[number]> }
  | {
      readonly status: "error";
      readonly match?: RouteMatch<Routes[number]>;
      readonly error: unknown;
      readonly errorComponent?: RouteComponentSpecifier;
    };

export interface RouterNavigationState<Routes extends readonly RouteDefinition[]> {
  readonly status: NavigationStatus;
  readonly current: RouteMatch<Routes[number]> | undefined;
  readonly pending: RouteMatch<Routes[number]> | undefined;
  readonly data: unknown;
  readonly error: unknown;
}

export interface RouterHistoryAdapter {
  readonly push: (url: string, leaving: RouterHistorySnapshot) => void | Promise<void>;
  readonly replace: (url: string, leaving: RouterHistorySnapshot) => void | Promise<void>;
}

export interface RouterHistorySnapshot {
  readonly scroll?: unknown;
  readonly focus?: unknown;
}

export interface RouterFocusAdapter {
  readonly capture?: () => unknown;
  readonly restore?: (snapshot: unknown) => void | Promise<void>;
  readonly focusMain?: (match: RouteMatch) => void | Promise<void>;
}

export interface RouterScrollAdapter {
  readonly capture?: () => unknown;
  readonly restore?: (snapshot: unknown) => void | Promise<void>;
  readonly top?: (match: RouteMatch) => void | Promise<void>;
}

export interface RouterNavigationOptions<Routes extends readonly RouteDefinition[]> {
  readonly initial?: string | URL;
  readonly history?: RouterHistoryAdapter;
  readonly focus?: RouterFocusAdapter;
  readonly scroll?: RouterScrollAdapter;
  readonly announce?: (message: string, match: RouteMatch<Routes[number]>) => void | Promise<void>;
  /** Maximum redirect chain before failing closed. @default 16 */
  readonly maxRedirects?: number;
}

export interface Router<Routes extends readonly RouteDefinition[]> {
  readonly matcher: RouterMatcher<Routes>;
  readonly state: () => RouterNavigationState<Routes>;
  readonly navigate: (
    location: RouteLocation<Routes>,
    options?: NavigationOptions,
  ) => Promise<NavigationResult<Routes>>;
  readonly beforeEach: (guard: NavigationGuard<Routes>) => () => void;
  readonly subscribe: (listener: (state: RouterNavigationState<Routes>) => void) => () => void;
  readonly cancel: (reason?: unknown) => void;
}

/** Source row used by the file-route generator. */
export interface FileRouteSource {
  readonly file: RouteComponentSpecifier;
  readonly name?: string;
  readonly path?: string;
  readonly meta?: RouteMeta;
  readonly loader?: RouteLoader<RouteParams, unknown>;
  readonly redirect?: RouteRedirect<RouteParams>;
  readonly errorComponent?: RouteComponentSpecifier;
}

/** Matcher produced by createRouterMatcher. */
export interface RouterMatcher<Routes extends readonly RouteDefinition[]> {
  readonly routes: Routes;
  readonly match: (input: string | URL) => RouteMatch<Routes[number]> | undefined;
  readonly manifest: () => RouteManifest;
  readonly resolve: <Name extends RouteNames<Routes>>(
    name: Name,
    params: RouteParamsForName<Routes, Name>,
    options?: RouteResolveOptions,
  ) => string;
}

/** Literal names available in a route table. */
export type RouteNames<Routes extends readonly RouteDefinition[]> = Routes[number]["name"];

/** Route record selected by literal name. */
export type RouteByName<
  Routes extends readonly RouteDefinition[],
  Name extends RouteNames<Routes>,
> = Extract<Routes[number], { readonly name: Name }>;

/** Params required by the route with `Name`. */
export type RouteParamsForName<
  Routes extends readonly RouteDefinition[],
  Name extends RouteNames<Routes>,
> =
  RouteByName<Routes, Name> extends RouteDefinition<string, infer Path>
    ? Simplify<RoutePathParams<Path>>
    : never;

/** Param object inferred from a path pattern. */
export type RoutePathParams<Path extends string> = Path extends `${infer Head}/${infer Tail}`
  ? Simplify<SegmentParams<Head> & RoutePathParams<Tail>>
  : SegmentParams<Path>;

export type RouteParams = Readonly<Record<string, string | undefined>>;
type SegmentParams<Segment extends string> = Segment extends `:${infer Name}?`
  ? { readonly [Key in CleanParamName<Name>]?: string }
  : Segment extends `:${infer Name}`
    ? { readonly [Key in CleanParamName<Name>]: string }
    : Segment extends `*${infer Name}`
      ? { readonly [Key in CleanParamName<Name>]: string }
      : {};
type CleanParamName<Name extends string> = Name extends "" ? never : Name;
export type Simplify<T> = { readonly [Key in keyof T]: T[Key] } & {};

export interface CompiledRoute<Route extends RouteDefinition = RouteDefinition> {
  readonly route: Route;
  readonly pattern: RegExp;
  readonly params: readonly string[];
}
