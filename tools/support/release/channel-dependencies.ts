// Dependency ranges between release channels.
//
// - A hard dependency (`dependencies` or `optionalDependencies`) on another Vize package stays
//   in the package's own channel or points at v1 alpha. A v1 alpha install therefore never pulls
//   in a v0, early or preview product, and those products can build on v1 alpha.
// - Within a channel, and from any channel onto v1 alpha, a hard dependency uses `workspace:*`,
//   which the release stamps to the exact version it publishes. Native platform binaries use the
//   `native-binaries` catalog.
// - A peer dependency on another Vize package uses an open `>=` lower bound, so either side can
//   release without stamping the other. Within one channel it may instead use `workspace:*`.
//
// `CHANNEL_DEPENDENCY_EXCEPTIONS` lists the edges that predate this policy and stay because
// removing them would change an existing install path. A new edge cannot join silently, and an
// exception that no longer matches a manifest fails as stale.

import { npmProduct, productChannel, type ReleaseChannel } from "./product-channels.ts";

export type DependencyField = "dependencies" | "optionalDependencies" | "peerDependencies";

export interface ChannelDependency {
  readonly from: string;
  readonly field: DependencyField;
  readonly to: string;
  readonly range: string;
}

export interface ChannelDependencyException extends ChannelDependency {
  readonly reason: string;
}

export const CHANNEL_DEPENDENCY_EXCEPTIONS: readonly ChannelDependencyException[] = [
  {
    from: "@vizejs/nuxt",
    field: "dependencies",
    to: "@vizejs/vite-plugin-musea",
    range: "workspace:*",
    reason: "The Nuxt module installs the Musea gallery integration for existing projects.",
  },
  {
    from: "@vizejs/vite-plugin-musea",
    field: "peerDependencies",
    to: "@vizejs/vite-plugin",
    range: "workspace:*",
    reason: "Musea shares the exact Vite plugin build it renders through.",
  },
];

const FIELDS: readonly DependencyField[] = [
  "dependencies",
  "optionalDependencies",
  "peerDependencies",
];

export interface DependencyManifest {
  readonly name: string;
  readonly dependencies?: Readonly<Record<string, string>>;
  readonly optionalDependencies?: Readonly<Record<string, string>>;
  readonly peerDependencies?: Readonly<Record<string, string>>;
}

/** Every dependency edge between two registered Vize npm packages. */
export function channelDependencies(manifests: readonly DependencyManifest[]): ChannelDependency[] {
  const edges: ChannelDependency[] = [];
  for (const manifest of manifests)
    for (const field of FIELDS)
      for (const [to, range] of Object.entries(manifest[field] ?? {}))
        if (npmProduct(to) !== undefined) edges.push({ from: manifest.name, field, to, range });
  return edges;
}

function channelOf(name: string): ReleaseChannel {
  const product = npmProduct(name);
  if (product === undefined) throw new Error(`${name} is not in the release channel registry`);
  return productChannel(product);
}

function sameEdge(left: ChannelDependency, right: ChannelDependency): boolean {
  return (
    left.from === right.from &&
    left.field === right.field &&
    left.to === right.to &&
    left.range === right.range
  );
}

/** Why one edge breaks the channel dependency policy, or undefined when it follows it. */
export function channelDependencyViolation(edge: ChannelDependency): string | undefined {
  const from = channelOf(edge.from);
  const to = channelOf(edge.to);
  const label = `${edge.from} (${from}) ${edge.field} ${edge.to} (${to}) "${edge.range}"`;
  if (edge.field === "peerDependencies") {
    if (/^>=\d+\.\d+\.\d+$/u.test(edge.range)) return undefined;
    if (from === to && edge.range === "workspace:*") return undefined;
    return from === to
      ? `${label}: use workspace:* or an open >= lower bound`
      : `${label}: a peer across channels takes an open >= lower bound`;
  }
  if (from !== to && to !== "v1-alpha")
    return `${label}: a hard dependency stays in its channel or points at v1 alpha`;
  const expected = edge.range.startsWith("catalog:") ? "catalog:native-binaries" : "workspace:*";
  return edge.range === expected ? undefined : `${label}: use ${expected}`;
}

/** Policy violations and stale exceptions over a set of package manifests. */
export function channelDependencyProblems(manifests: readonly DependencyManifest[]): string[] {
  const edges = channelDependencies(manifests);
  const problems: string[] = [];
  for (const edge of edges) {
    const violation = channelDependencyViolation(edge);
    if (violation && !CHANNEL_DEPENDENCY_EXCEPTIONS.some((exception) => sameEdge(exception, edge)))
      problems.push(violation);
  }
  for (const exception of CHANNEL_DEPENDENCY_EXCEPTIONS) {
    const edge = edges.find((candidate) => sameEdge(candidate, exception));
    if (!edge) problems.push(`stale exception: ${exception.from} -> ${exception.to}`);
    else if (!channelDependencyViolation(edge))
      problems.push(`unneeded exception: ${exception.from} -> ${exception.to}`);
  }
  return problems;
}
