#!/usr/bin/env bash

set -euo pipefail

readonly graph_crates=(vize_atelier_dom vize_atelier_ssr vize_atelier_vapor)

for crate in "${graph_crates[@]}"; do
  cargo check -p "$crate" --no-default-features --features graph

  resolved="$({
    cargo tree \
      -p "$crate" \
      --no-default-features \
      --features graph \
      --edges normal \
      --prefix none \
      --format '{p}'
  })"

  internal="$({
    printf '%s\n' "$resolved" \
      | awk '$1 ~ /^vize_/ { print $1 }' \
      | sort -u
  })"
  expected="$({
    printf '%s\n' "$crate" vize_atlas vize_carton vize_rendu \
      | sort -u
  })"

  if [[ "$internal" != "$expected" ]]; then
    printf 'graph-only dependency boundary failed for %s\n' "$crate" >&2
    printf 'expected internal crates:\n%s\n' "$expected" >&2
    printf 'resolved internal crates:\n%s\n' "$internal" >&2
    exit 1
  fi

  forbidden="$({
    printf '%s\n' "$resolved" \
      | awk '$1 ~ /^(vize_(armature|atelier_core|croquis|relief)|oxc_)/ { print $1 }' \
      | sort -u
  })"
  if [[ -n "$forbidden" ]]; then
    printf 'graph-only dependency tree for %s contains frontend crates:\n%s\n' \
      "$crate" "$forbidden" >&2
    exit 1
  fi
done

# The neutral script frontend may depend on Atlas and Flow, but never on Vue
# template syntax, semantic products, compilers, or their consumers.
module_internal="$({
  cargo tree -p vize_module --edges normal --prefix none --format '{p}' \
    | awk '$1 ~ /^vize_/ { print $1 }' \
    | sort -u
})"
module_expected="$({
  printf '%s\n' vize_module vize_atlas vize_carton vize_flow | sort -u
})"
if [[ "$module_internal" != "$module_expected" ]]; then
  printf 'neutral module dependency boundary failed\nexpected:\n%s\nresolved:\n%s\n' \
    "$module_expected" "$module_internal" >&2
  exit 1
fi

# Atelier Core is a transform/emission helper, not a syntax/parser/allocator
# facade. Canary public APIs must name the owning crate directly.
core_public_facades="$({
  rg -n \
    '^pub use vize_(relief|armature|carton)' \
    crates/vize_atelier_core/src/lib.rs || true
})"

if [[ -n "$core_public_facades" ]]; then
  printf 'vize_atelier_core publicly re-exports owned syntax/parser/allocator APIs:\n%s\n' \
    "$core_public_facades" >&2
  exit 1
fi

readonly owned_modules='errors|options|parser|tokenizer'
readonly owned_symbols='Allocator|AllocBox|AllocVec|CloneIn|Parser|parse|parse_with_options|parse_with_options_and_invalid_html_self_closing|parse_with_options_and_template_syntax|CompilerError|CompilerResult|ErrorCode|BindingMetadata|BindingType|CodegenMode|CodegenOptions|CompilerOptions|ParseMode|ParserOptions|TemplateSyntaxMode|TextMode|TransformOptions|WhitespaceStrategy|AttributeNode|CommentNode|CompoundExpressionChild|CompoundExpressionNode|ConstantType|DirectiveNode|ElementNode|ElementType|ExpressionNode|ForNode|ForParseResult|IfBranchNode|IfNode|ImportItem|InterpolationNode|JsExpression|Namespace|NodeType|Position|PropNode|RootNode|RuntimeHelper|SimpleExpressionNode|SourceLocation|TemplateChildNode|TextCallContent|TextCallNode|TextNode'

core_facade_routes="$({
  rg -n -U \
    --glob 'crates/*/src/**/*.rs' \
    --glob '!crates/vize_atelier_core/**' \
    "vize_atelier_core::(${owned_modules})\\b|vize_atelier_core::(${owned_symbols})\\b|use vize_atelier_core::\\{[^;]*\\b(${owned_symbols})\\b[^;]*\\};" \
    crates || true
})"

if [[ -n "$core_facade_routes" ]]; then
  printf 'production consumers route owned syntax/parser/allocator APIs through vize_atelier_core:\n%s\n' \
    "$core_facade_routes" >&2
  printf 'import Relief, Armature, or Carton directly; Atelier Core owns only shared transform/emission helpers\n' >&2
  exit 1
fi

# Public standalone compile/compileVapor hosts must query the raw-template
# product. parseTemplate remains the only explicit parser endpoint.
raw_template_bypasses="$({
  rg -n \
    'compile_template_with_|compile_vapor_with_|compile_ssr_with_|compile_internal|vize_atelier_core::' \
    crates/vize_vitrine/src/napi/template.rs \
    crates/vize_vitrine/src/wasm/compiler.rs || true
})"

if [[ -n "$raw_template_bypasses" ]]; then
  printf 'standalone template hosts bypass vize_atelier_template Atlas products:\n%s\n' \
    "$raw_template_bypasses" >&2
  exit 1
fi

# Public FFI lint hosts must enter through the persistent Atlas report root.
# Direct Linter calls are reserved for Patina's implementation and unit tests.
ffi_lint_bypasses="$({
  rg -n \
    '\.lint_(sfc|template|standalone_html)\(' \
    crates/vize_vitrine/src || true
})"

if [[ -n "$ffi_lint_bypasses" ]]; then
  printf 'NAPI/WASM lint hosts bypass PatinaDocumentReportProduct:\n%s\n' \
    "$ffi_lint_bypasses" >&2
  exit 1
fi

# Every production CLI lint input, including autofix revalidation, must query
# the persistent Atlas report root. Direct Patina calls belong only in tests.
cli_lint_bypasses="$({
  rg -n \
    'direct_outcome|lint_source|\.lint_(sfc|jsx|script|template|standalone_html)\(' \
    crates/vize/src/commands/lint/artifact_graph.rs \
    crates/vize/src/commands/lint/pipeline.rs \
    crates/vize/src/commands/lint/fix.rs || true
})"

if [[ -n "$cli_lint_bypasses" ]]; then
  printf 'CLI lint hosts bypass PatinaDocumentReportProduct:\n%s\n' \
    "$cli_lint_bypasses" >&2
  exit 1
fi
