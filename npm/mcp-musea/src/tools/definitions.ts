/**
 * MCP tool definitions for Musea.
 *
 * Declares the schema (name, description, input parameters) for each tool
 * exposed by the MCP server: component analysis, registry, code generation,
 * documentation, and design tokens.
 */

export const toolDefinitions = [
  // --- Component analysis ---------------------------------------------------
  {
    name: "analyze_component",
    description:
      "Statically analyze a Vue SFC to extract its props and emits. Accepts a Vue component path directly, or an art-file reference that resolves to the linked component source.",
    inputSchema: {
      type: "object" as const,
      properties: {
        path: {
          type: "string",
          description:
            "Path to the .vue component file or .art.vue file (relative to project root)",
        },
        title: {
          type: "string",
          description:
            "Resolve an art file by its display title, then analyze its component source",
        },
        component: {
          type: "string",
          description:
            "Resolve an art file by its component reference or component basename, then analyze it",
        },
        query: {
          type: "string",
          description: "Fuzzy-search an art file, then analyze the linked component source",
        },
        ref: {
          type: "string",
          description: "Generic art-file reference: path, title, component name, or search text",
        },
      },
      required: [],
    },
  },
  {
    name: "get_palette",
    description:
      "Derive an interactive props palette (control types, defaults, ranges, options) for a component described by an Art file. Falls back to SFC analysis when native palette inference is sparse.",
    inputSchema: {
      type: "object" as const,
      properties: {
        path: {
          type: "string",
          description: "Path to the .art.vue file (relative to project root)",
        },
        title: {
          type: "string",
          description: "Resolve an art file by title instead of path",
        },
        component: {
          type: "string",
          description: "Resolve an art file by component reference or component basename",
        },
        query: {
          type: "string",
          description: "Fuzzy-search an art file before generating the palette",
        },
        ref: {
          type: "string",
          description: "Generic art-file reference: path, title, component name, or search text",
        },
      },
      required: [],
    },
  },

  // --- Component registry ---------------------------------------------------
  {
    name: "list_components",
    description:
      "List components registered in the design system. Returns titles, categories, tags, status, variant names, and related resource URIs.",
    inputSchema: {
      type: "object" as const,
      properties: {
        category: { type: "string", description: "Filter by category" },
        tag: { type: "string", description: "Filter by tag" },
        status: {
          type: "string",
          enum: ["draft", "ready", "deprecated"],
          description: "Filter by status badge",
        },
        component: {
          type: "string",
          description: "Filter by component reference or component basename",
        },
        limit: {
          type: "number",
          description: "Maximum number of components to return (default: all)",
        },
        includeVariants: {
          type: "boolean",
          description: "Include per-variant metadata in the result (default: false)",
        },
        sortBy: {
          type: "string",
          enum: ["title", "category", "status", "variants"],
          description: "Sort order for the result list (default: title)",
        },
      },
    },
  },
  {
    name: "get_component",
    description:
      "Get full details of a design-system component: metadata, variants, source-component analysis, palette data, documentation, and related resource URIs.",
    inputSchema: {
      type: "object" as const,
      properties: {
        path: {
          type: "string",
          description: "Path to the .art.vue file (relative to project root)",
        },
        title: {
          type: "string",
          description: "Resolve an art file by display title instead of path",
        },
        component: {
          type: "string",
          description: "Resolve an art file by component reference or component basename",
        },
        query: {
          type: "string",
          description: "Fuzzy-search an art file before loading details",
        },
        ref: {
          type: "string",
          description: "Generic art-file reference: path, title, component name, or search text",
        },
        includeAnalysis: {
          type: "boolean",
          description: "Include resolved component props/emits analysis (default: true)",
        },
        includePalette: {
          type: "boolean",
          description: "Include inferred palette data (default: true)",
        },
        includeDocumentation: {
          type: "boolean",
          description: "Include generated Markdown docs inline (default: false)",
        },
      },
      required: [],
    },
  },
  {
    name: "get_variant",
    description:
      "Retrieve a single variant (template and metadata) from a component, resolving the component by path, title, component name, or fuzzy query.",
    inputSchema: {
      type: "object" as const,
      properties: {
        path: { type: "string", description: "Path to the .art.vue file" },
        title: { type: "string", description: "Resolve an art file by title" },
        component: {
          type: "string",
          description: "Resolve an art file by component reference or component basename",
        },
        query: {
          type: "string",
          description: "Fuzzy-search an art file before looking up the variant",
        },
        ref: {
          type: "string",
          description: "Generic art-file reference: path, title, component name, or search text",
        },
        variant: { type: "string", description: "Variant name" },
        includeAnalysis: {
          type: "boolean",
          description: "Include resolved component props/emits analysis (default: false)",
        },
      },
      required: ["variant"],
    },
  },
  {
    name: "search_components",
    description:
      "Ranked full-text search over component titles, descriptions, categories, tags, component names, and variant names.",
    inputSchema: {
      type: "object" as const,
      properties: {
        query: { type: "string", description: "Search query" },
        category: { type: "string", description: "Restrict matches to one category" },
        tag: { type: "string", description: "Restrict matches to one tag" },
        status: {
          type: "string",
          enum: ["draft", "ready", "deprecated"],
          description: "Restrict matches to one status",
        },
        component: {
          type: "string",
          description: "Restrict matches to a component reference/basename before searching",
        },
        limit: {
          type: "number",
          description: "Maximum number of results to return (default: 10)",
        },
      },
      required: ["query"],
    },
  },
  {
    name: "recommend_components",
    description:
      "Intent-oriented component recommendation. Useful when the user describes a task or UX goal rather than knowing exact component names.",
    inputSchema: {
      type: "object" as const,
      properties: {
        task: { type: "string", description: "Intent or UI task to solve" },
        category: { type: "string", description: "Optional category filter" },
        tag: { type: "string", description: "Optional tag filter" },
        status: {
          type: "string",
          enum: ["draft", "ready", "deprecated"],
          description: "Optional status filter",
        },
        component: {
          type: "string",
          description: "Optional component reference/basename filter",
        },
        limit: {
          type: "number",
          description: "Maximum number of recommendations to return (default: 5)",
        },
      },
      required: ["task"],
    },
  },

  // --- Code generation ------------------------------------------------------
  {
    name: "generate_variants",
    description:
      "Analyze a Vue component's props and auto-generate an .art.vue file containing appropriate variant combinations (default, boolean toggles, enum values, etc.).",
    inputSchema: {
      type: "object" as const,
      properties: {
        componentPath: {
          type: "string",
          description: "Path to the .vue component file (relative to project root)",
        },
        maxVariants: {
          type: "number",
          description: "Maximum number of variants to generate (default: 20)",
        },
        includeDefault: {
          type: "boolean",
          description: "Include a default variant (default: true)",
        },
        includeBooleanToggles: {
          type: "boolean",
          description: "Generate variants that toggle each boolean prop (default: true)",
        },
        includeEnumVariants: {
          type: "boolean",
          description: "Generate one variant per enum/union value (default: true)",
        },
      },
      required: ["componentPath"],
    },
  },
  {
    name: "generate_csf",
    description:
      "Convert an .art.vue file into Storybook CSF 3.0 code for integration with existing Storybook setups.",
    inputSchema: {
      type: "object" as const,
      properties: {
        path: { type: "string", description: "Path to the .art.vue file" },
        title: { type: "string", description: "Resolve an art file by title" },
        component: {
          type: "string",
          description: "Resolve an art file by component reference or component basename",
        },
        query: {
          type: "string",
          description: "Fuzzy-search an art file before converting to CSF",
        },
        ref: {
          type: "string",
          description: "Generic art-file reference: path, title, component name, or search text",
        },
      },
      required: [],
    },
  },

  // --- Documentation --------------------------------------------------------
  {
    name: "generate_docs",
    description:
      "Generate Markdown documentation for a design-system component from its .art.vue definition.",
    inputSchema: {
      type: "object" as const,
      properties: {
        path: {
          type: "string",
          description: "Path to the .art.vue file (relative to project root)",
        },
        title: { type: "string", description: "Resolve an art file by title" },
        component: {
          type: "string",
          description: "Resolve an art file by component reference or component basename",
        },
        query: {
          type: "string",
          description: "Fuzzy-search an art file before generating docs",
        },
        ref: {
          type: "string",
          description: "Generic art-file reference: path, title, component name, or search text",
        },
        includeSource: {
          type: "boolean",
          description: "Embed source code in the output (default: false)",
        },
        includeTemplates: {
          type: "boolean",
          description: "Embed variant templates in the output (default: false)",
        },
      },
      required: [],
    },
  },
  {
    name: "generate_catalog",
    description:
      "Produce a single Markdown catalog covering every component in the design system, grouped by category.",
    inputSchema: {
      type: "object" as const,
      properties: {
        includeSource: {
          type: "boolean",
          description: "Embed source code in the catalog (default: false)",
        },
        includeTemplates: {
          type: "boolean",
          description: "Embed variant templates in the catalog (default: false)",
        },
      },
    },
  },

  // --- Design tokens --------------------------------------------------------
  {
    name: "get_tokens",
    description:
      "Read design tokens (colors, spacing, typography, etc.) from a Style Dictionary-compatible JSON file or directory. Auto-detects common paths if not specified.",
    inputSchema: {
      type: "object" as const,
      properties: {
        tokensPath: {
          type: "string",
          description:
            "Path to tokens JSON file or directory (relative to project root). Auto-detects tokens/, design-tokens/, or style-dictionary/ if omitted.",
        },
        format: {
          type: "string",
          enum: ["json", "markdown"],
          description: "Output format (default: json)",
        },
      },
    },
  },
  {
    name: "search_tokens",
    description:
      "Search flattened design tokens by token name, category path, value, or description. Much more practical than loading the full token tree for large systems.",
    inputSchema: {
      type: "object" as const,
      properties: {
        query: { type: "string", description: "Search query" },
        tokensPath: {
          type: "string",
          description:
            "Path to tokens JSON file or directory (relative to project root). Auto-detects common locations if omitted.",
        },
        type: {
          type: "string",
          description: "Optional token type filter, e.g. color, dimension, typography",
        },
        limit: {
          type: "number",
          description: "Maximum number of matches to return (default: 20)",
        },
      },
      required: ["query"],
    },
  },
];
