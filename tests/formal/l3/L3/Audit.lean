import Lean

/-!
Proof audit for the Impeto package. `#audit_impeto_theorems` walks every
theorem imported from an `Impeto.*` module and fails elaboration unless its
transitive dependencies stay within Lean's standard foundations. A proof hole,
a compiler-trusted decision procedure or a new postulate therefore breaks
`lake build`, independently of the textual scan in the CI workflow.
-/

namespace Impeto.Audit
open Lean Elab Command

def trusted : List Name := [``propext, ``Quot.sound, ``Classical.choice]

def impetoTheorems (env : Environment) : Array Name :=
  env.constants.map₁.fold (init := #[]) fun found name info =>
    match info, env.getModuleIdxFor? name with
    | .thmInfo _, some index =>
        if (`Impeto).isPrefixOf (env.header.moduleNames[index.toNat]!) then found.push name
        else found
    | _, _ => found

elab "#audit_impeto_theorems " minimum:num : command => do
  let names := impetoTheorems (← getEnv)
  for name in names do
    let foundations ← liftCoreM (collectAxioms name)
    for dependency in foundations do
      unless trusted.contains dependency do
        throwError m!"{name} depends on untrusted foundation {dependency}"
  if names.size < minimum.getNat then
    throwError m!"expected at least {minimum.getNat} audited Impeto theorems, found {names.size}"
  logInfo m!"audited {names.size} Impeto theorems"

end Impeto.Audit
