import assert from "node:assert/strict";

import { getCompileOptionsForRequest } from "./state.ts";

assert.deepEqual(
  getCompileOptionsForRequest(
    {
      isProduction: false,
      mergedOptions: { vapor: true },
    },
    false,
  ),
  {
    sourceMap: true,
    ssr: false,
    vapor: true,
    customRenderer: false,
    templateSyntax: "standard",
    styleTrim: true,
  },
  "Client requests should keep Vapor enabled when the plugin is configured for it",
);

assert.equal(
  getCompileOptionsForRequest(
    {
      isProduction: true,
      mergedOptions: { templateSyntax: "quirks" },
    },
    false,
  ).templateSyntax,
  "quirks",
  "Request compile options should preserve configured template syntax",
);

assert.deepEqual(
  getCompileOptionsForRequest(
    {
      isProduction: true,
      mergedOptions: { vapor: true },
    },
    true,
  ),
  {
    sourceMap: false,
    ssr: true,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
    styleTrim: true,
  },
  "SSR requests should continue to use the VDOM compiler while client builds hydrate with Vapor",
);

assert.deepEqual(
  getCompileOptionsForRequest(
    {
      isProduction: false,
      mergedOptions: {
        experimentalInTagComments: true,
        experimentalPatternedTemplate: true,
        experimentalServerScript: true,
      },
    },
    false,
  ),
  {
    sourceMap: true,
    ssr: false,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
    styleTrim: true,
    experimentalInTagComments: true,
    experimentalPatternedTemplate: true,
    experimentalServerScript: true,
  },
  "Request compile options should pass experimental flags to native compilation",
);
