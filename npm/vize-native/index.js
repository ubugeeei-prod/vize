/* eslint-disable */
// @ts-nocheck

const nativeBinding = require("./native-binding");
const tokenNativeBinding = {
  buildDesignTokenMap: nativeBinding.buildDesignTokenMap,
  findDependentDesignTokens: nativeBinding.findDependentDesignTokens,
  flattenDesignTokenCategories: nativeBinding.flattenDesignTokenCategories,
  generateDesignTokensMarkdown: nativeBinding.generateDesignTokensMarkdown,
  parseDesignTokensFromJson: nativeBinding.parseDesignTokensFromJson,
  parseDesignTokensFromPath: nativeBinding.parseDesignTokensFromPath,
  resolveDesignTokenReferences: nativeBinding.resolveDesignTokenReferences,
  validateDesignTokenReference: nativeBinding.validateDesignTokenReference,
};

function parseJsonResult(value) {
  return JSON.parse(value);
}

function stringifyJsonArg(value) {
  return JSON.stringify(value) ?? "null";
}

function buildDesignTokenMap(categories) {
  return parseJsonResult(tokenNativeBinding.buildDesignTokenMap(stringifyJsonArg(categories)));
}

function findDependentDesignTokens(tokenMap, targetPath) {
  return tokenNativeBinding.findDependentDesignTokens(stringifyJsonArg(tokenMap), targetPath);
}

function flattenDesignTokenCategories(categories) {
  return parseJsonResult(
    tokenNativeBinding.flattenDesignTokenCategories(stringifyJsonArg(categories)),
  );
}

function generateDesignTokensMarkdown(categories, generatedAt) {
  return tokenNativeBinding.generateDesignTokensMarkdown(stringifyJsonArg(categories), generatedAt);
}

function parseDesignTokensFromJson(source) {
  return parseJsonResult(tokenNativeBinding.parseDesignTokensFromJson(source));
}

function parseDesignTokensFromPath(tokensPath) {
  return parseJsonResult(tokenNativeBinding.parseDesignTokensFromPath(tokensPath));
}

function resolveDesignTokenReferences(categories) {
  return parseJsonResult(
    tokenNativeBinding.resolveDesignTokenReferences(stringifyJsonArg(categories)),
  );
}

function validateDesignTokenReference(tokenMap, reference, selfPath) {
  return parseJsonResult(
    tokenNativeBinding.validateDesignTokenReference(
      stringifyJsonArg(tokenMap),
      reference,
      selfPath,
    ),
  );
}

module.exports = nativeBinding;
module.exports.artToCsf = nativeBinding.artToCsf;
module.exports.applyViteDefineReplacements = nativeBinding.applyViteDefineReplacements;
module.exports.buildDesignTokenMap = buildDesignTokenMap;
module.exports.classifyVitePluginRequest = nativeBinding.classifyVitePluginRequest;
module.exports.chunkVitePrecompileFiles = nativeBinding.chunkVitePrecompileFiles;
module.exports.collectSfcTemplateAssetUrls = nativeBinding.collectSfcTemplateAssetUrls;
module.exports.compile = nativeBinding.compile;
module.exports.compileCss = nativeBinding.compileCss;
module.exports.compileSfc = nativeBinding.compileSfc;
module.exports.compileSfcBatch = nativeBinding.compileSfcBatch;
module.exports.compileSfcBatchWithResults = nativeBinding.compileSfcBatchWithResults;
module.exports.compileVapor = nativeBinding.compileVapor;
module.exports.createViteBareImportBases = nativeBinding.createViteBareImportBases;
module.exports.createViteBareImportCandidates = nativeBinding.createViteBareImportCandidates;
module.exports.createViteVirtualId = nativeBinding.createViteVirtualId;
module.exports.detectViteHmrUpdateType = nativeBinding.detectViteHmrUpdateType;
module.exports.diffVitePrecompileFiles = nativeBinding.diffVitePrecompileFiles;
module.exports.extractSfcCustomBlocks = nativeBinding.extractSfcCustomBlocks;
module.exports.extractSfcSrcInfo = nativeBinding.extractSfcSrcInfo;
module.exports.extractSfcStyleBlocks = nativeBinding.extractSfcStyleBlocks;
module.exports.findDependentDesignTokens = findDependentDesignTokens;
module.exports.flattenDesignTokenCategories = flattenDesignTokenCategories;
module.exports.formatSfc = nativeBinding.formatSfc;
module.exports.fromViteVirtualId = nativeBinding.fromViteVirtualId;
module.exports.generateArtCatalog = nativeBinding.generateArtCatalog;
module.exports.generateArtDoc = nativeBinding.generateArtDoc;
module.exports.generateArtDocsBatch = nativeBinding.generateArtDocsBatch;
module.exports.generateArtPalette = nativeBinding.generateArtPalette;
module.exports.generateDesignTokensMarkdown = generateDesignTokensMarkdown;
module.exports.generateDeclaration = nativeBinding.generateDeclaration;
module.exports.generateSfcScopeId = nativeBinding.generateSfcScopeId;
module.exports.generateVariants = nativeBinding.generateVariants;
module.exports.generateViteHmrCode = nativeBinding.generateViteHmrCode;
module.exports.getPatinaRules = nativeBinding.getPatinaRules;
module.exports.getTypeCheckCapabilities = nativeBinding.getTypeCheckCapabilities;
module.exports.hasSfcScopedStyle = nativeBinding.hasSfcScopedStyle;
module.exports.hasVitePrecompileFileMetadataChanged =
  nativeBinding.hasVitePrecompileFileMetadataChanged;
module.exports.hasViteHmrChanges = nativeBinding.hasViteHmrChanges;
module.exports.isBuiltinViteDefine = nativeBinding.isBuiltinViteDefine;
module.exports.isSfcImportableAssetUrl = nativeBinding.isSfcImportableAssetUrl;
module.exports.isViteBareSpecifier = nativeBinding.isViteBareSpecifier;
module.exports.lint = nativeBinding.lint;
module.exports.lintPatinaSfc = nativeBinding.lintPatinaSfc;
module.exports.normalizeViteFsIdForBuild = nativeBinding.normalizeViteFsIdForBuild;
module.exports.normalizeViteCssModuleFilename = nativeBinding.normalizeViteCssModuleFilename;
module.exports.normalizeViteDevMiddlewareUrl = nativeBinding.normalizeViteDevMiddlewareUrl;
module.exports.normalizeVitePrecompileBatchSize = nativeBinding.normalizeVitePrecompileBatchSize;
module.exports.normalizeViteRequireBase = nativeBinding.normalizeViteRequireBase;
module.exports.normalizeViteResolvedVuePath = nativeBinding.normalizeViteResolvedVuePath;
module.exports.normalizeViteVirtualVueModuleId = nativeBinding.normalizeViteVirtualVueModuleId;
module.exports.parseArt = nativeBinding.parseArt;
module.exports.parseDesignTokensFromJson = parseDesignTokensFromJson;
module.exports.parseDesignTokensFromPath = parseDesignTokensFromPath;
module.exports.parseSfc = nativeBinding.parseSfc;
module.exports.parseTemplate = nativeBinding.parseTemplate;
module.exports.resolveDesignTokenReferences = resolveDesignTokenReferences;
module.exports.resolveViteAliasRequest = nativeBinding.resolveViteAliasRequest;
module.exports.resolveViteCssImports = nativeBinding.resolveViteCssImports;
module.exports.resolveViteRelativeImport = nativeBinding.resolveViteRelativeImport;
module.exports.resolveViteVuePath = nativeBinding.resolveViteVuePath;
module.exports.rewriteViteDynamicTemplateImports = nativeBinding.rewriteViteDynamicTemplateImports;
module.exports.rewriteViteImportMetaGlobBase = nativeBinding.rewriteViteImportMetaGlobBase;
module.exports.rewriteViteStaticAssetUrls = nativeBinding.rewriteViteStaticAssetUrls;
module.exports.runCli = nativeBinding.runCli;
module.exports.scopeViteCssForPipeline = nativeBinding.scopeViteCssForPipeline;
module.exports.transformViteCssVarsForPipeline = nativeBinding.transformViteCssVarsForPipeline;
module.exports.shouldApplyViteDefineInVirtualModule =
  nativeBinding.shouldApplyViteDefineInVirtualModule;
module.exports.splitViteIdQuery = nativeBinding.splitViteIdQuery;
module.exports.stripSfcScopedCssComments = nativeBinding.stripSfcScopedCssComments;
module.exports.toViteBrowserImportPrefix = nativeBinding.toViteBrowserImportPrefix;
module.exports.typeCheck = nativeBinding.typeCheck;
module.exports.typeCheckBatch = nativeBinding.typeCheckBatch;
module.exports.validateDesignTokenReference = validateDesignTokenReference;
module.exports.wrapSfcScopedPreprocessorStyle = nativeBinding.wrapSfcScopedPreprocessorStyle;
