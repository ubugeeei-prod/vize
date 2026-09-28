import Lean

/-!
Proof audit for the L3 package. `#audit_l3_theorems` walks every
theorem imported from an `L3.*` module and fails elaboration unless its
transitive dependencies stay within Lean's standard foundations. A proof hole,
a compiler-trusted decision procedure or a new postulate therefore breaks
`lake build`, independently of the textual scan in the CI workflow.
-/

namespace L3.Audit
open Lean Elab Command

def trusted : List Name := [``propext, ``Quot.sound, ``Classical.choice]

def l3Theorems (env : Environment) : Array Name :=
  env.constants.map₁.fold (init := #[]) fun found name info =>
    match info, env.getModuleIdxFor? name with
    | .thmInfo _, some index =>
        if (`L3).isPrefixOf (env.header.moduleNames[index.toNat]!) then found.push name
        else found
    | _, _ => found

elab "#audit_l3_theorems " minimum:num : command => do
  let names := l3Theorems (← getEnv)
  for name in names do
    let foundations ← liftCoreM (collectAxioms name)
    for dependency in foundations do
      unless trusted.contains dependency do
        throwError m!"{name} depends on untrusted foundation {dependency}"
  if names.size < minimum.getNat then
    throwError m!"expected at least {minimum.getNat} audited L3 theorems, found {names.size}"
  logInfo m!"audited {names.size} L3 theorems"

end L3.Audit
