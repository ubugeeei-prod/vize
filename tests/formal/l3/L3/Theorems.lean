import L3.Audit
import L3.LatticeLaws
import L3.ScheduleLaws
import L3.IncrementalLaws
import L3.CheckerLaws

/-!
Every P3-15 theorem module is imported here, so the audit below sees the whole
proof surface. The executable imports this module; `lake build` fails when a
theorem is missing, holed, or relies on anything beyond standard foundations.
-/

#audit_l3_theorems 200
