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

export interface ArtStyleBlock {
  content: string;
  lang: string;
  scoped: boolean;
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
  /** Whether <script setup> state is isolated for each variant */
  scriptSetupIsolated?: boolean;
  /** Whether file has regular script */
  hasScript: boolean;
  /** Number of style blocks */
  styleCount: number;
  /** Raw style contents retained for ArtFileInfo compatibility. */
  styleBlocks?: string[];
  /** Style metadata used to delegate each block to Vite's CSS pipeline. */
  styleBlockDetails?: ArtStyleBlock[];
  /** Whether this art comes from an inline <art> block in a regular .vue file */
  isInline?: boolean;
  /** For inline art: absolute path to the host .vue component file */
  componentPath?: string;
  /** For inline art: position of its <art> block in the native custom-block list. */
  inlineArtBlockIndex?: number;
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
