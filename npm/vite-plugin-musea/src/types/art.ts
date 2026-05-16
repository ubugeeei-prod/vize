/**
 * Art file metadata.
 */
export interface ArtMetadata {
  title: string;
  description?: string;
  component?: string;
  category?: string;
  tags: string[];
  status: "draft" | "ready" | "deprecated";
  order?: number;
  /** Additional DOM events to capture in the Actions panel, e.g. ["mousemove"] */
  actionEvents?: string[];
}

/**
 * Art variant definition.
 */
export interface ArtVariant {
  name: string;
  template: string;
  isDefault: boolean;
  skipVrt: boolean;
  args?: Record<string, unknown>;
}

/**
 * Parsed Art file information.
 */
export interface ArtFileInfo {
  /** Absolute file path */
  path: string;
  /** Art metadata */
  metadata: ArtMetadata;
  /** Variant definitions */
  variants: ArtVariant[];
  /** Whether file has script setup */
  hasScriptSetup: boolean;
  /** Raw content of the <script setup> block */
  scriptSetupContent?: string;
  /** Whether file has regular script */
  hasScript: boolean;
  /** Number of style blocks */
  styleCount: number;
  /** Raw CSS blocks extracted from the Art file for preview injection */
  styleBlocks?: string[];
  /** Whether this art comes from an inline <art> block in a regular .vue file */
  isInline?: boolean;
  /** For inline art: absolute path to the host .vue component file */
  componentPath?: string;
}

/**
 * CSF output from Art transformation.
 */
export interface CsfOutput {
  /** Generated CSF code */
  code: string;
  /** Suggested filename */
  filename: string;
}
