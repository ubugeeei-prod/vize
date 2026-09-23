import Impeto.Incremental

/-!
P3-15 IVM theorems for the reference update machine that TS-29 executes.
For every successful update, including keyed and positional `v-for`
reconciliation:

* the patched tree displays exactly the freshly rendered view (`update_tree`),
  so incremental maintenance equals recompute-from-scratch;
* each top-level node either keeps the identity of the previous sibling with
  its S3 address or receives a fresh identity from the allocator window;
* the allocator advances by exactly the number of elements with no retained
  address (`reconcile_allocations`), so a keyed reorder or payload patch
  allocates nothing and work is linear in the inserted delta.
-/

namespace Impeto.Incremental

/-- Identity of the first previous sibling carrying this S3 address. -/
def retainedIdentity? (previous : List Node) (owner : String) : Option Nat :=
  match previous.find? (fun node => address node == some owner) with
  | some (.element _ identity _ _ _ _ _) => some identity
  | _ => none

mutual
/-- Elements the update must create: views whose address has no retained node. -/
def allocationsOne (previous : List Node) : View -> Nat
  | .text _ => 0
  | .element owner _ _ _ _ children =>
      match previous.find? (fun node => address node == some owner) with
      | some (.element _ _ _ _ _ _ oldChildren) => allocationsAll oldChildren children
      | _ => 1 + allocationsAll [] children

def allocationsAll (previous : List Node) : List View -> Nat
  | [] => 0
  | view :: rest => allocationsOne previous view + allocationsAll previous rest
end

theorem retain_ok {previous : List Node} {owner tag : String} {next identity after : Nat}
    {children : List Node} (h : retain previous owner tag next = .ok (identity, children, after)) :
    (retainedIdentity? previous owner = some identity ∧ after = next ∧
      ∃ oldAddress oldTag attrs disabled event,
        previous.find? (fun node => address node == some owner) =
          some (.element oldAddress identity oldTag attrs disabled event children)) ∨
    (retainedIdentity? previous owner = none ∧ identity = next ∧ after = next + 1 ∧ children = []) := by
  unfold retain at h
  unfold retainedIdentity?
  split at h
  · next oldAddress oldIdentity oldTag attrs disabled event oldChildren found =>
      split at h
      · simp only [Except.ok.injEq, Prod.mk.injEq] at h
        obtain ⟨rfl, rfl, rfl⟩ := h
        exact .inl ⟨by rw [found], rfl, oldAddress, oldTag, attrs, disabled, event, found⟩
      · contradiction
  · next notElement =>
      simp only [Except.ok.injEq, Prod.mk.injEq] at h
      obtain ⟨rfl, rfl, rfl⟩ := h
      refine .inr ⟨?_, rfl, rfl, rfl⟩
      split
      · next found => exact absurd found (notElement _ _ _ _ _ _ _)
      · rfl

section
variable {fuel : Nat}

theorem siblings_erase_of
    (hr : ∀ previous desired next nodes after,
      reconcile fuel previous desired next = .ok (nodes, after) -> eraseAll nodes = desired) :
    ∀ (desired : List View) (previous : List Node) (seen : List String) (next : Nat)
      (nodes : List Node) (after : Nat),
      siblings fuel previous seen desired next = .ok (nodes, after) -> eraseAll nodes = desired := by
  intro desired
  induction desired with
  | nil =>
      intro previous seen next nodes after h
      simp only [siblings, Except.ok.injEq, Prod.mk.injEq] at h
      obtain ⟨rfl, -⟩ := h
      rfl
  | cons view rest ih =>
      intro previous seen next nodes after h
      cases view with
      | text value =>
          simp only [siblings] at h
          split at h
          · contradiction
          · rename_i restNodes restNext hrest
            simp only [Except.ok.injEq, Prod.mk.injEq] at h
            obtain ⟨rfl, -⟩ := h
            simp [eraseAll, erase, ih _ _ _ _ _ hrest]
      | element owner tag attrs disabled event children =>
          simp only [siblings] at h
          split at h
          · contradiction
          split at h
          · contradiction
          rename_i identity oldChildren n1 _
          split at h
          · contradiction
          rename_i updated n2 hchildren
          split at h
          · contradiction
          rename_i restNodes n3 hrest
          simp only [Except.ok.injEq, Prod.mk.injEq] at h
          obtain ⟨rfl, -⟩ := h
          simp [eraseAll, erase, hr _ _ _ _ _ hchildren, ih _ _ _ _ _ hrest]

theorem siblings_allocations_of
    (hr : ∀ previous desired next nodes after,
      reconcile fuel previous desired next = .ok (nodes, after) ->
        after = next + allocationsAll previous desired) :
    ∀ (desired : List View) (previous : List Node) (seen : List String) (next : Nat)
      (nodes : List Node) (after : Nat),
      siblings fuel previous seen desired next = .ok (nodes, after) ->
        after = next + allocationsAll previous desired := by
  intro desired
  induction desired with
  | nil =>
      intro previous seen next nodes after h
      simp only [siblings, Except.ok.injEq, Prod.mk.injEq] at h
      obtain ⟨-, rfl⟩ := h
      simp [allocationsAll]
  | cons view rest ih =>
      intro previous seen next nodes after h
      cases view with
      | text value =>
          simp only [siblings] at h
          split at h
          · contradiction
          · rename_i restNodes restNext hrest
            simp only [Except.ok.injEq, Prod.mk.injEq] at h
            obtain ⟨-, rfl⟩ := h
            simp [allocationsAll, allocationsOne, ih _ _ _ _ _ hrest]
      | element owner tag attrs disabled event children =>
          simp only [siblings] at h
          split at h
          · contradiction
          split at h
          · contradiction
          rename_i identity oldChildren n1 hretain
          split at h
          · contradiction
          rename_i updated n2 hchildren
          split at h
          · contradiction
          rename_i restNodes n3 hrest
          simp only [Except.ok.injEq, Prod.mk.injEq] at h
          obtain ⟨-, rfl⟩ := h
          rw [ih _ _ _ _ _ hrest, hr _ _ _ _ _ hchildren]
          simp only [allocationsAll, allocationsOne]
          rcases retain_ok hretain with ⟨_, rfl, oldAddress, oldTag, attrs', disabled', event', found⟩ |
            ⟨_, _, rfl, rfl⟩
          · simp only [found]; omega
          · split
            · next found => simp_all [retainedIdentity?]
            · omega

end

theorem reconcile_erase : ∀ (fuel : Nat) (previous : List Node) (desired : List View)
    (next : Nat) (nodes : List Node) (after : Nat),
    reconcile fuel previous desired next = .ok (nodes, after) -> eraseAll nodes = desired
  | 0, _, _, _, _, _, h => by rw [reconcile] at h; cases h
  | fuel + 1, previous, desired, next, nodes, after, h => by
      simp only [reconcile] at h
      exact siblings_erase_of (reconcile_erase fuel) desired previous [] next nodes after h

/-- Allocation is exactly the inserted delta. -/
theorem reconcile_allocations : ∀ (fuel : Nat) (previous : List Node) (desired : List View)
    (next : Nat) (nodes : List Node) (after : Nat),
    reconcile fuel previous desired next = .ok (nodes, after) ->
      after = next + allocationsAll previous desired
  | 0, _, _, _, _, _, h => by rw [reconcile] at h; cases h
  | fuel + 1, previous, desired, next, nodes, after, h => by
      simp only [reconcile] at h
      exact siblings_allocations_of (reconcile_allocations fuel) desired previous [] next nodes after h

theorem siblings_advance {fuel : Nat} {desired : List View} {previous nodes : List Node}
    {seen : List String} {next after : Nat}
    (h : siblings fuel previous seen desired next = .ok (nodes, after)) : next ≤ after := by
  have := siblings_allocations_of (reconcile_allocations fuel) desired previous seen next nodes after h
  omega

/-- Keyed retention: every top-level element keeps the identity of the previous
sibling with its S3 address, or takes a fresh identity from `[next, after)`. -/
theorem siblings_identity (fuel : Nat) : ∀ (desired : List View) (previous : List Node)
    (seen : List String) (next : Nat) (nodes : List Node) (after : Nat),
    siblings fuel previous seen desired next = .ok (nodes, after) ->
      ∀ node, node ∈ nodes -> ∀ owner identity tag attrs disabled event children,
        node = .element owner identity tag attrs disabled event children ->
          retainedIdentity? previous owner = some identity ∨
            (retainedIdentity? previous owner = none ∧ next ≤ identity ∧ identity < after) := by
  intro desired
  induction desired with
  | nil =>
      intro previous seen next nodes after h node mem
      simp only [siblings, Except.ok.injEq, Prod.mk.injEq] at h
      obtain ⟨rfl, -⟩ := h
      cases mem
  | cons view rest ih =>
      intro previous seen next nodes after h node mem
      cases view with
      | text value =>
          simp only [siblings] at h
          split at h
          · contradiction
          · rename_i restNodes restNext hrest
            simp only [Except.ok.injEq, Prod.mk.injEq] at h
            obtain ⟨rfl, rfl⟩ := h
            cases mem with
            | head => intro _ _ _ _ _ _ _ same; cases same
            | tail _ mem => exact ih _ _ _ _ _ hrest node mem
      | element owner tag attrs disabled event children =>
          simp only [siblings] at h
          split at h
          · contradiction
          split at h
          · contradiction
          rename_i identity oldChildren n1 hretain
          split at h
          · contradiction
          rename_i updated n2 hchildren
          split at h
          · contradiction
          rename_i restNodes n3 hrest
          simp only [Except.ok.injEq, Prod.mk.injEq] at h
          obtain ⟨rfl, rfl⟩ := h
          have childAdvance : n1 ≤ n2 := by
            have := reconcile_allocations fuel oldChildren children n1 updated n2 hchildren
            omega
          have restAdvance := siblings_advance hrest
          cases mem with
          | head =>
              intro owner' identity' _ _ _ _ _ same
              cases same
              rcases retain_ok hretain with ⟨found, _⟩ | ⟨missing, rfl, rfl, -⟩
              · exact .inl found
              · exact .inr ⟨missing, Nat.le_refl _, by omega⟩
          | tail _ mem =>
              intro owner' identity' tag' attrs' disabled' event' children' same
              rcases ih _ _ _ _ _ hrest node mem owner' identity' tag' attrs' disabled' event'
                children' same with found | ⟨missing, low, high⟩
              · exact .inl found
              · refine .inr ⟨missing, ?_, high⟩
                rcases retain_ok hretain with ⟨_, rfl, -⟩ | ⟨_, _, rfl, -⟩ <;> omega

/-- Incremental maintenance equals recompute-from-scratch on every accepted update. -/
theorem update_tree {fuel : Nat} {state result : State} {view : List View}
    (h : update fuel state view = .ok result) :
    eraseAll result.roots = view ∧ tree result = View.tree view := by
  unfold update at h
  split at h
  · contradiction
  · rename_i roots next hreconcile
    dsimp only at h
    split at h
    · simp only [Except.ok.injEq] at h
      subst h
      have erased := reconcile_erase fuel state.roots view state.nextIdentity roots next hreconcile
      exact ⟨erased, by simp [tree, erased]⟩
    · contradiction

/-- A keyed reorder or payload patch whose every address survives allocates nothing. -/
theorem update_without_insertions {fuel : Nat} {state result : State} {view : List View}
    (h : update fuel state view = .ok result) (survives : allocationsAll state.roots view = 0) :
    result.nextIdentity = state.nextIdentity := by
  unfold update at h
  split at h
  · contradiction
  · rename_i roots next hreconcile
    dsimp only at h
    split at h
    · simp only [Except.ok.injEq] at h
      subst h
      have := reconcile_allocations fuel state.roots view state.nextIdentity roots next hreconcile
      simp [this, survives]
    · contradiction

end Impeto.Incremental
