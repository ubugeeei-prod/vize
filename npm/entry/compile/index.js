// The compile slice of @vizejs/native; every value is the native export itself.
import native from "@vizejs/native";

export const compile = native.compile;
export const compileVapor = native.compileVapor;
export const compileSfc = native.compileSfc;
export const compileSfcBatch = native.compileSfcBatch;
export const compileSfcBatchWithResults = native.compileSfcBatchWithResults;
export const compileCss = native.compileCss;
export const parseSfc = native.parseSfc;
export const parseTemplate = native.parseTemplate;
export const parseTemplateAst = native.parseTemplateAst;
export const analyzeSfc = native.analyzeSfc;
export const compileWithOutputPlugins = native.compileWithOutputPlugins;
export const compileWithTransformPlugins = native.compileWithTransformPlugins;
export const applyOutputPlugins = native.applyOutputPlugins;
export const extractSfcCustomBlocks = native.extractSfcCustomBlocks;
export const extractSfcSrcInfo = native.extractSfcSrcInfo;
export const extractSfcStyleBlocks = native.extractSfcStyleBlocks;
export const generateSfcScopeId = native.generateSfcScopeId;
export const hasSfcScopedStyle = native.hasSfcScopedStyle;
export const parseCssAst = native.parseCssAst;
export const printCssAst = native.printCssAst;
