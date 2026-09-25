import assert from "node:assert/strict";

import { compileFile } from "../compiler.ts";
import { buildCompileBatchOptions } from "../compile-options.ts";
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
    templateComments: true,
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

assert.equal(
  getCompileOptionsForRequest(
    {
      isProduction: false,
      mergedOptions: {
        whitespace: "condense",
        template: { compilerOptions: { whitespace: "preserve" } },
      },
    },
    false,
  ).whitespace,
  "preserve",
  "The plugin-vue template option should override the Vize compiler default",
);

assert.equal(
  getCompileOptionsForRequest(
    {
      isProduction: false,
      mergedOptions: { whitespace: "vue2-line-breaks" },
    },
    false,
  ).whitespace,
  "vue2-line-breaks",
  "Vize's migration mode should reach on-demand compilation",
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
    templateComments: false,
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
        experimentalSelfComponent: true,
        experimentalStrictSlotChildren: true,
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
    templateComments: true,
    styleTrim: true,
    experimentalInTagComments: true,
    experimentalPatternedTemplate: true,
    experimentalSelfComponent: true,
    experimentalStrictSlotChildren: true,
    experimentalServerScript: true,
  },
  "Request compile options should pass experimental flags to native compilation",
);

const commentedSfc = '<template>\n  <!-- note -->\n  <div class="a">x</div>\n</template>';
const filePath = "/src/Commented.vue";

function compileWithComments(isProduction: boolean, comments?: boolean): string {
  const mergedOptions = {
    sourceMap: false,
    ...(comments === undefined ? {} : { template: { compilerOptions: { comments } } }),
  };
  const requestOptions = getCompileOptionsForRequest({ isProduction, mergedOptions }, false);
  assert.equal(
    buildCompileBatchOptions(requestOptions).templateComments,
    requestOptions.templateComments,
    "precompile and on-demand compilation must use the same comment option",
  );
  return compileFile(filePath, new Map(), requestOptions, commentedSfc).code;
}

const developmentCode = compileWithComments(false);
assert.match(developmentCode, /createCommentVNode\(" note "\)/);
assert.match(developmentCode, /_Fragment/);

const productionCode = compileWithComments(true);
assert.doesNotMatch(productionCode, / note /);
assert.doesNotMatch(productionCode, /_Fragment/);

assert.doesNotMatch(compileWithComments(false, false), / note /);
assert.match(compileWithComments(true, true), /createCommentVNode\(" note "\)/);
