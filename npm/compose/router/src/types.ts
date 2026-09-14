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
}

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
