"""Give the three serial PR report jobs sparse checkouts of exactly their runtime."""
import sys

root = sys.argv[1]
checkout = "      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6\n"
rust = """        with:
          fetch-depth: 1
          # The report gate needs only its script closure and the bootstrap action.
          sparse-checkout-cone-mode: false
          sparse-checkout: |
            /package.json
            /.github/actions/cache-vp-bootstrap/
            /tools/support/compat/github/require-rust-tier.ts
            /tools/support/compat/github/require-needs-success.ts
"""
source = """        with:
          fetch-depth: 1
          # The source gate needs only its script closure, never product sources.
          sparse-checkout-cone-mode: false
          sparse-checkout: |
            /package.json
            /tools/support/compat/github/canonical-corpus-selection.mjs
            /tools/support/compat/github/plan-source-checks.ts
            /tools/support/compat/github/require-needs-success.ts
"""
for file, job, block in [
    (".github/workflows/pr-rust-products.yml", "  rust-product-report:\n", rust),
    (".github/workflows/pr-rust-checks.yml", "  rust-source-report:\n", rust),
    (".github/workflows/pr-source-checks.yml", "  source-report:\n", source),
]:
    path = f"{root}/{file}"
    text = open(path).read()
    start = text.index(job)
    at = text.index(checkout, start) + len(checkout)
    assert text.count(job) == 1 and not text[at:].startswith("        with:"), file
    text = text[:at] + block + text[at:]
    open(path, "w").write(text)
    print("sparse", file)
