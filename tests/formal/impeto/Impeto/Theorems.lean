import Impeto.Audit
import Impeto.LatticeLaws
import Impeto.ScheduleLaws
import Impeto.IncrementalLaws
import Impeto.CheckerLaws

/-!
Every P3-15 theorem module is imported here, so the audit below sees the whole
proof surface. The executable imports this module; `lake build` fails when a
theorem is missing, holed, or relies on anything beyond standard foundations.
-/

#audit_impeto_theorems 200
