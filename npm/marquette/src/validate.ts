import type {
  ApplicationMarquette,
  MARQUETTE_FORMAT_VERSION,
  MarquetteEnvironment,
  MarquetteRoute,
  MarquetteTarget,
  RenderingMode,
} from "./model.js";

/** Severity of an application-marquette diagnostic. */
export type MarquetteDiagnosticSeverity = "error" | "warning";

/** Stable, source-addressable application-marquette diagnostic. */
export interface MarquetteDiagnostic {
  /** Stable machine-readable diagnostic code. */
  readonly code: `VIZE_MARQUETTE_${string}`;
  /** Severity used by CLI, editor, test, and CI consumers. */
  readonly severity: MarquetteDiagnosticSeverity;
  /** JSON-style path into the authored marquette. */
  readonly path: string;
  /** Human-readable explanation and next action. */
  readonly message: string;
}

const IDENTIFIER = /^[a-z0-9][a-z0-9._-]*$/;
const SUPPORTED_FORMAT_VERSION: typeof MARQUETTE_FORMAT_VERSION = 1;

/**
 * Validates a complete application marquette.
 *
 * Diagnostics cover structural, reference, capability, target, and dependency
 * invariants. Results are sorted by path, code, and message so identical input
 * produces stable editor, CLI, test, and CI output.
 *
 * This function does not mutate the authored object and does not throw.
 */
export function validateApplicationMarquette(
  marquette: ApplicationMarquette,
): MarquetteDiagnostic[] {
  const diagnostics: MarquetteDiagnostic[] = [];
  const environments = marquette.environments ?? [];
  const backends = marquette.backends ?? [];
  const protocols = marquette.protocols ?? [];
  const routes = marquette.routes ?? [];
  const targets = new Set(marquette.targets ?? []);
  const capabilityIds = new Set(Object.keys(marquette.capabilities ?? {}));

  if ((marquette.formatVersion ?? SUPPORTED_FORMAT_VERSION) !== SUPPORTED_FORMAT_VERSION) {
    diagnostics.push(
      error("VIZE_MARQUETTE_001", "formatVersion", "unsupported marquette format version"),
    );
  }

  validateIdentifier(marquette.application, "application", "VIZE_MARQUETTE_002", diagnostics);

  for (const [key, capability] of Object.entries(marquette.capabilities ?? {})) {
    const path = contractPath("capabilities", key);
    validateIdentifier(key, path, "VIZE_MARQUETTE_003", diagnostics);
    if (key !== capability.id) {
      diagnostics.push(
        error("VIZE_MARQUETTE_004", path, "capability map key must equal capability.id"),
      );
    }
    const version = capability.version ?? 1;
    if (!Number.isInteger(version) || version < 1) {
      diagnostics.push(
        error("VIZE_MARQUETTE_005", path, "capability version must be greater than zero"),
      );
    }
    if (capability.description.trim().length === 0) {
      diagnostics.push(
        error("VIZE_MARQUETTE_024", path, "capability description must not be empty"),
      );
    }
  }

  const environmentIds = collectUniqueIds("environments", environments, diagnostics);
  const backendIds = collectUniqueIds("backends", backends, diagnostics);
  const protocolIds = collectUniqueIds("protocols", protocols, diagnostics);
  collectUniqueIds("routes", routes, diagnostics);

  for (const environment of environments) {
    const path = contractPath("environments", environment.id);
    if (!targets.has(environment.target)) {
      diagnostics.push(
        error("VIZE_MARQUETTE_007", path, "environment target must be declared in targets"),
      );
    }
    for (const dependency of [...new Set(environment.dependsOn ?? [])].sort()) {
      if (dependency === environment.id) {
        diagnostics.push(error("VIZE_MARQUETTE_008", path, "environment cannot depend on itself"));
      } else if (!environmentIds.has(dependency)) {
        diagnostics.push(
          error("VIZE_MARQUETTE_009", path, "environment dependency does not exist"),
        );
      }
    }
    validateCapabilities(path, environment.capabilities, capabilityIds, diagnostics);

    if (
      environment.consumer === "client" &&
      (environment.runtime === "rust" ||
        environment.runtime === "go" ||
        environment.runtime === "jvm")
    ) {
      diagnostics.push(
        warning(
          "VIZE_MARQUETTE_010",
          path,
          "client environment uses a server-oriented runtime; declare an adapter capability if this is intentional",
        ),
      );
    }
  }

  validateEnvironmentCycles(environments, diagnostics);

  for (const backend of backends) {
    const path = contractPath("backends", backend.id);
    if (backend.environment != null) {
      const environment = environments.find((candidate) => candidate.id === backend.environment);
      if (environment == null) {
        diagnostics.push(error("VIZE_MARQUETTE_011", path, "backend environment does not exist"));
      } else if (environment.consumer !== "server") {
        diagnostics.push(
          error("VIZE_MARQUETTE_012", path, "backend environment must be a server consumer"),
        );
      }
    }
    validateCapabilities(path, backend.capabilities, capabilityIds, diagnostics);
  }

  for (const protocol of protocols) {
    const path = contractPath("protocols", protocol.id);
    if (!backendIds.has(protocol.backend)) {
      diagnostics.push(error("VIZE_MARQUETTE_013", path, "protocol backend does not exist"));
    }
    validateCapabilities(path, protocol.capabilities, capabilityIds, diagnostics);
  }

  const routePaths = new Map<string, string>();
  for (const route of routes) {
    const path = contractPath("routes", route.id);
    if (!route.path.startsWith("/")) {
      diagnostics.push(error("VIZE_MARQUETTE_014", path, "route path must start with /"));
    }
    const environment = environments.find((candidate) => candidate.id === route.environment);
    if (environment == null) {
      diagnostics.push(error("VIZE_MARQUETTE_015", path, "route environment does not exist"));
    }
    if (route.backend != null && !backendIds.has(route.backend)) {
      diagnostics.push(error("VIZE_MARQUETTE_016", path, "route backend does not exist"));
    }
    if (route.protocol != null) {
      const protocol = protocols.find((candidate) => candidate.id === route.protocol);
      if (!protocolIds.has(route.protocol) || protocol == null) {
        diagnostics.push(error("VIZE_MARQUETTE_017", path, "route protocol does not exist"));
      } else if (route.backend != null && protocol.backend !== route.backend) {
        diagnostics.push(
          error(
            "VIZE_MARQUETTE_018",
            path,
            "route protocol and backend must refer to the same service",
          ),
        );
      }
    }
    if (environment != null && !renderingMatchesTarget(route, environment.target)) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_023",
          path,
          "rendering mode is not compatible with the route environment target",
        ),
      );
    }
    validateCapabilities(path, route.capabilities, capabilityIds, diagnostics);

    const routeKey = `${route.environment}\u0000${route.path}`;
    const previous = routePaths.get(routeKey);
    if (previous != null) {
      diagnostics.push(
        error("VIZE_MARQUETTE_019", path, `route path is already used by route "${previous}"`),
      );
    }
    routePaths.set(routeKey, route.id);
  }

  for (const target of targets) {
    if (!environments.some((environment) => environment.target === target)) {
      diagnostics.push(
        warning("VIZE_MARQUETTE_020", "targets", "declared target has no environment"),
      );
    }
  }

  return diagnostics.sort(compareDiagnostics);
}

function collectUniqueIds(
  collection: string,
  values: readonly { readonly id: string }[],
  diagnostics: MarquetteDiagnostic[],
): Set<string> {
  const ids = new Set<string>();
  for (const value of values) {
    const path = contractPath(collection, value.id);
    validateIdentifier(value.id, path, "VIZE_MARQUETTE_006", diagnostics);
    if (ids.has(value.id)) {
      diagnostics.push(
        error("VIZE_MARQUETTE_006", path, "identifier must be unique within its collection"),
      );
    }
    ids.add(value.id);
  }
  return ids;
}

function validateIdentifier(
  id: string,
  path: string,
  code: `VIZE_MARQUETTE_${string}`,
  diagnostics: MarquetteDiagnostic[],
): void {
  if (!IDENTIFIER.test(id)) {
    diagnostics.push(
      error(
        code,
        path,
        "identifier must use lowercase ASCII letters, digits, dash, underscore, or dot",
      ),
    );
  }
}

function validateCapabilities(
  path: string,
  capabilities: readonly string[] | undefined,
  declared: ReadonlySet<string>,
  diagnostics: MarquetteDiagnostic[],
): void {
  for (const capability of [...new Set(capabilities ?? [])].sort()) {
    if (!declared.has(capability)) {
      diagnostics.push(error("VIZE_MARQUETTE_021", path, "referenced capability is not declared"));
    }
  }
}

function validateEnvironmentCycles(
  environments: readonly MarquetteEnvironment[],
  diagnostics: MarquetteDiagnostic[],
): void {
  const graph = new Map(environments.map((value) => [value.id, value.dependsOn ?? []]));
  const visiting = new Set<string>();
  const visited = new Set<string>();

  for (const id of [...graph.keys()].sort()) {
    if (hasCycle(id, graph, visiting, visited)) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_022",
          contractPath("environments", id),
          "environment dependency graph must be acyclic",
        ),
      );
    }
  }
}

function hasCycle(
  id: string,
  graph: ReadonlyMap<string, readonly string[]>,
  visiting: Set<string>,
  visited: Set<string>,
): boolean {
  if (visited.has(id)) return false;
  if (visiting.has(id)) return true;
  visiting.add(id);
  const cyclic = [...(graph.get(id) ?? [])]
    .sort()
    .some((dependency) => graph.has(dependency) && hasCycle(dependency, graph, visiting, visited));
  visiting.delete(id);
  visited.add(id);
  return cyclic;
}

function renderingMatchesTarget(route: MarquetteRoute, target: MarquetteTarget): boolean {
  const targetRendering: Partial<Record<RenderingMode, MarquetteTarget>> = {
    native: "native",
    desktop: "desktop",
    terminal: "terminal",
  };
  return (targetRendering[route.rendering] ?? "web") === target;
}

function compareDiagnostics(left: MarquetteDiagnostic, right: MarquetteDiagnostic): number {
  return (
    compareText(left.path, right.path) ||
    compareText(left.code, right.code) ||
    compareText(left.message, right.message)
  );
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

function contractPath(collection: string, id: string): string {
  return `${collection}.${id}`;
}

function error(
  code: `VIZE_MARQUETTE_${string}`,
  path: string,
  message: string,
): MarquetteDiagnostic {
  return { code, severity: "error", path, message };
}

function warning(
  code: `VIZE_MARQUETTE_${string}`,
  path: string,
  message: string,
): MarquetteDiagnostic {
  return { code, severity: "warning", path, message };
}
