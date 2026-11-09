// Shapes shared by the consumer migration surface scanner and its renderers.

export type SurfaceGroup = "stage" | "old" | "raw";
export type NameKind = "preferred" | "compat" | "legacy" | "raw";
export type SiteMode = "source" | "manifest" | "test";

export interface Surface {
  id: string;
  label: string;
  group: SurfaceGroup;
  names: string[];
  preferredNames?: string[];
  compatNames?: string[];
}

export interface ConsumerEntry {
  path?: string;
  crate?: string;
  include?: (relPath: string) => boolean;
  exclude?: (relPath: string) => boolean;
}

export interface Consumer {
  id: string;
  label: string;
  scope: string;
  entries: ConsumerEntry[];
}

export interface SurfaceSite {
  relPath: string;
  line: number;
  mode: SiteMode;
  surfaceId: string;
  group: SurfaceGroup;
  matchedName: string;
  nameKind: NameKind;
}

export interface FileRow {
  relPath: string;
  mode: SiteMode;
  firstLine: number;
  total: number;
  surfaceCounts: Record<string, number>;
  nameKindCounts: Record<NameKind, number>;
  surfaceNameCounts: Record<string, Record<string, number>>;
}

export interface ConsumerSummary extends Consumer {
  fileCount: number;
  surfaceFileCount: number;
  sites: SurfaceSite[];
  groupCounts: Record<SurfaceGroup, number>;
  modeCounts: Record<SiteMode, number>;
  surfaceCounts: Record<string, number>;
  nameKindCounts: Record<NameKind, number>;
  fileRows: FileRow[];
}

export interface ConsumerScan {
  surfaces: readonly Surface[];
  consumers: readonly ConsumerSummary[];
}
