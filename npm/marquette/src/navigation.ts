import type { ApplicationMarquette, MarquetteRoute, RouteId } from "./model.js";

/** Scalar accepted by one authored route-path parameter. */
export type NavigationParameter = string | number;

type DeclaredRoute<Marquette extends ApplicationMarquette> = Marquette extends {
  readonly routes: readonly (infer Route extends MarquetteRoute)[];
}
  ? Route
  : never;

type RouteFor<Marquette extends ApplicationMarquette, Id extends RouteId<Marquette>> = Extract<
  DeclaredRoute<Marquette>,
  { readonly id: Id }
>;

type RoutePath<Marquette extends ApplicationMarquette, Id extends RouteId<Marquette>> = RouteFor<
  Marquette,
  Id
>["path"];

type SegmentParameter<Segment extends string> = Segment extends `:${infer Name}`
  ? Name extends ""
    ? never
    : Name
  : Segment extends `*${infer Name}`
    ? Name extends ""
      ? never
      : Name
    : never;

type PathParameter<Path extends string> = Path extends `${infer Segment}/${infer Rest}`
  ? SegmentParameter<Segment> | PathParameter<Rest>
  : SegmentParameter<Path>;

/** Exact route parameters inferred from full `:name` and `*name` path segments. */
export type NavigationParameters<
  Marquette extends ApplicationMarquette,
  Id extends RouteId<Marquette>,
> = {
  readonly [Name in PathParameter<RoutePath<Marquette, Id>>]: NavigationParameter;
};

type NavigationArguments<Marquette extends ApplicationMarquette, Id extends RouteId<Marquette>> = [
  PathParameter<RoutePath<Marquette, Id>>,
] extends [never]
  ? []
  : [parameters: NavigationParameters<Marquette, Id>];

/** Target passed to a host adapter after route parameters are encoded. */
export type ResolvedNavigationTarget<
  Marquette extends ApplicationMarquette,
  Id extends RouteId<Marquette> = RouteId<Marquette>,
> =
  Id extends RouteId<Marquette>
    ? {
        /** Authored route identifier. */
        readonly route: Id;
        /** Environment that owns the target view. */
        readonly environment: RouteFor<Marquette, Id>["environment"];
        /** Resolved path with every parameter percent-encoded. */
        readonly path: `/${string}`;
      }
    : never;

/** Programmatic navigation surface inferred from one application marquette. */
export interface MarquetteNavigator<Marquette extends ApplicationMarquette, Result> {
  /** Resolve and deliver one route to the target-specific host adapter. */
  readonly navigate: <const Id extends RouteId<Marquette>>(
    route: Id,
    ...arguments_: NavigationArguments<Marquette, Id>
  ) => Result;
}

/**
 * Creates a target-neutral, fully inferred programmatic navigator.
 *
 * A `:name` segment accepts one scalar and is encoded as one path segment. A
 * `*name` segment also accepts a scalar, but preserves `/` between its encoded
 * segments. Unknown routes, missing parameters, extra parameters, empty
 * strings, and non-finite numbers fail before the host adapter runs.
 */
export function createMarquetteNavigator<const Marquette extends ApplicationMarquette, Result>(
  marquette: Marquette,
  visit: (target: ResolvedNavigationTarget<Marquette>) => Result,
): MarquetteNavigator<Marquette, Result> {
  const routes = new Map<string, MarquetteRoute>(
    (marquette.routes ?? []).map((route) => [route.id, route]),
  );

  const navigate = (
    routeId: string,
    parameters: Readonly<Record<string, NavigationParameter>> = {},
  ): Result => {
    const route = routes.get(routeId);
    if (route == null) throw new Error(`Unknown navigation route "${routeId}".`);

    return visit({
      route: route.id,
      environment: route.environment,
      path: resolveNavigationPath(routeId, route.path, parameters),
    } as ResolvedNavigationTarget<Marquette>);
  };

  return {
    navigate: navigate as MarquetteNavigator<Marquette, Result>["navigate"],
  };
}

function resolveNavigationPath(
  routeId: string,
  template: `/${string}`,
  parameters: Readonly<Record<string, NavigationParameter>>,
): `/${string}` {
  const expected = parameterSegments(template);
  const expectedNames = new Set(expected.map(({ name }) => name));

  for (const { name } of expected) {
    if (!Object.hasOwn(parameters, name)) {
      throw new Error(`Missing navigation parameter "${name}" for route "${routeId}".`);
    }
  }

  const extra = Object.keys(parameters)
    .filter((name) => !expectedNames.has(name))
    .sort()[0];
  if (extra != null) {
    throw new Error(`Unexpected navigation parameter "${extra}" for route "${routeId}".`);
  }

  const segments = template.split("/").map((segment) => {
    const parameter = expected.find(({ authored }) => authored === segment);
    if (parameter == null) return segment;
    const value = parameters[parameter.name];
    return encodeParameter(routeId, parameter.name, value, parameter.rest);
  });
  return segments.join("/") as `/${string}`;
}

function parameterSegments(
  template: string,
): { readonly authored: string; readonly name: string; readonly rest: boolean }[] {
  return template
    .split("/")
    .filter((segment) => (segment.startsWith(":") || segment.startsWith("*")) && segment.length > 1)
    .map((authored) => ({
      authored,
      name: authored.slice(1),
      rest: authored.startsWith("*"),
    }));
}

function encodeParameter(routeId: string, name: string, value: unknown, rest: boolean): string {
  if (
    (typeof value !== "string" && typeof value !== "number") ||
    (typeof value === "string" && value.length === 0) ||
    (typeof value === "number" && !Number.isFinite(value))
  ) {
    throw new Error(`Invalid navigation parameter "${name}" for route "${routeId}".`);
  }
  const text = String(value);
  return rest ? text.split("/").map(encodeURIComponent).join("/") : encodeURIComponent(text);
}
