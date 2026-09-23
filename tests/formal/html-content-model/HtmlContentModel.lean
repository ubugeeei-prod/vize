/-
Davinci P4-11a, charter #36: the HTML content-model checker's core
(`crates/vize_patina/src/html_content_model/`) stated in Lean.

* `HtmlContentModel.Tri`   — the three-valued facts and their Kleene
  connectives, each sound for every concrete truth value it admits.
* `HtmlContentModel.Walk`  — the stack-of-open-elements walk every scope
  and loop scan is built from (`Chain::walk`), sound for every concrete
  stack the (possibly truncated) chain describes, stable as the context
  grows, and the all-dispatches verdict rule.

Every definition is structurally recursive, so Lean itself checks that the
checker is total; every verdict is a `Bool`/`DecidableEq` computation, so it
is decidable. CI rejects any proof escape in the package.
-/
import HtmlContentModel.Tri
import HtmlContentModel.Walk
