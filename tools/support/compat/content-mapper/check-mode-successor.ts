/** Producer lines introduced by the diagnostics-only projection, beside their predecessors. */
export const CHECK_MODE_PRODUCER_SUCCESSORS = [
  [
    "            template_use_navigation: !transform_options.diagnostics_only(),\n",
    "            template_use_navigation: true,\n",
  ],
  [
    "            preserve_event_navigation: !transform_options.diagnostics_only(),\n",
    "            preserve_event_navigation: true,\n",
  ],
  ["            omit_navigation_references: transform_options.diagnostics_only(),\n", ""],
] as const;
