//! Kani checks the production HTML content-model kernel directly: the
//! three-valued facts (`tri.rs`) and the stack-of-open-elements walk every
//! scope and loop scan is built from (`walk.rs`).
//!
//! These harnesses replace the Lean package `tests/formal/html-content-model`.
//! Each one names the Lean theorem it replaces. Unlike the Lean model, they
//! verify the shipped Rust source, not a restatement of it.

extern crate core;

mod tri;
mod walk;

pub use tri::Tri;

#[cfg(kani)]
mod proofs {
    use super::Tri;
    use super::walk::{Role, Step, classify, walk};

    /// The longest known chain a bounded harness explores.
    const FRAMES: usize = 4;
    /// The most ancestors a truncated context reveals in a bounded harness.
    const REVEALED: usize = 2;
    /// HTML, SVG and MathML.
    const NAMESPACES: usize = 3;

    /// A frame as the walk sees it: the namespaces it may have and its role
    /// under each one.
    #[derive(Clone, Copy)]
    struct Frame {
        namespaces: u8,
        roles: [Role; NAMESPACES],
    }

    impl Frame {
        fn has(self, namespace: usize) -> bool {
            self.namespaces & (1 << namespace) != 0
        }

        /// The roles under each possible namespace, in namespace order, as
        /// `Chain::walk` passes them to the kernel.
        fn possible_roles(&self) -> impl Iterator<Item = Role> + '_ {
            let frame = *self;
            (0..NAMESPACES)
                .filter(move |namespace| frame.has(*namespace))
                .map(move |namespace| frame.role(namespace))
        }

        fn role(self, namespace: usize) -> Role {
            match namespace {
                0 => self.roles[0],
                1 => self.roles[1],
                _ => self.roles[2],
            }
        }
    }

    fn any_tri() -> Tri {
        match kani::any::<u8>() % 3 {
            0 => Tri::No,
            1 => Tri::Maybe,
            _ => Tri::Yes,
        }
    }

    fn any_role() -> Role {
        match kani::any::<u8>() % 3 {
            0 => Role::Target,
            1 => Role::Stop,
            _ => Role::Pass,
        }
    }

    fn any_frame() -> Frame {
        let namespaces: u8 = kani::any();
        kani::assume(namespaces < 1 << NAMESPACES);
        Frame {
            namespaces,
            roles: [any_role(), any_role(), any_role()],
        }
    }

    fn any_frames<const N: usize>() -> ([Frame; N], usize) {
        let frames = [(); N].map(|()| any_frame());
        let len: usize = kani::any();
        kani::assume(len <= N);
        (frames, len)
    }

    /// A fact is consistent with a concrete truth value (Lean `Tri.admits`).
    fn admits(fact: Tri, concrete: bool) -> bool {
        match fact {
            Tri::Yes => concrete,
            Tri::No => !concrete,
            Tri::Maybe => true,
        }
    }

    /// The verdict below the outermost known frame: `Document` when nothing
    /// else is open, `Truncated` when anything may be.
    fn exhausted(truncated: bool) -> Tri {
        if truncated { Tri::Maybe } else { Tri::No }
    }

    /// The parser's own scan over a concrete stack, innermost last: every
    /// element has one namespace and so one role (Lean `run`).
    fn run(stack: &[Role]) -> bool {
        for role in stack.iter().rev() {
            match role {
                Role::Target => return true,
                Role::Stop => return false,
                Role::Pass => {}
            }
        }
        false
    }

    /// Lean `Tri.not_sound`, `Tri.and_sound` and `Tri.or_sound`.
    #[kani::proof]
    fn kleene_connectives_are_sound() {
        let (left, right) = (any_tri(), any_tri());
        let (a, b): (bool, bool) = (kani::any(), kani::any());
        kani::assume(admits(left, a) && admits(right, b));
        assert!(admits(left.not(), !a));
        assert!(admits(left.and(right), a && b));
        assert!(admits(left.or(right), a || b));
    }

    /// Lean `Tri.or_eq_not_and_not`.
    #[kani::proof]
    fn disjunction_is_the_de_morgan_dual_of_conjunction() {
        let (left, right) = (any_tri(), any_tri());
        assert_eq!(left.or(right), left.not().and(right.not()).not());
    }

    /// Lean `Tri.admits_unique`.
    #[kani::proof]
    fn a_definite_fact_admits_exactly_one_value() {
        let fact = any_tri();
        let (a, b): (bool, bool) = (kani::any(), kani::any());
        kani::assume(fact != Tri::Maybe && admits(fact, a) && admits(fact, b));
        assert_eq!(a, b);
    }

    /// The inductive step of Lean `walk_sound`, for a frame of any shape:
    /// whatever one frame decides holds under each namespace it may have,
    /// and a frame passes only when it passes under each of them. The walk
    /// applies this step to one frame after another, so the step carries
    /// soundness to chains of every length.
    #[kani::proof]
    #[kani::unwind(5)]
    fn one_frame_decides_soundly_under_each_possible_namespace() {
        let frame = any_frame();
        let namespace: usize = kani::any();
        kani::assume(namespace < NAMESPACES && frame.has(namespace));
        let role = frame.role(namespace);
        match classify(frame.possible_roles()) {
            Step::Decided(Tri::Yes) => assert_eq!(role, Role::Target),
            Step::Decided(Tri::No) => assert_eq!(role, Role::Stop),
            Step::Decided(Tri::Maybe) => {}
            Step::Pass => assert_eq!(role, Role::Pass),
        }
    }

    /// A frame with no possible namespace passes, as the Lean `walk` skips it.
    #[kani::proof]
    #[kani::unwind(5)]
    fn a_frame_without_a_namespace_passes() {
        let frame = Frame {
            namespaces: 0,
            ..any_frame()
        };
        assert_eq!(classify(frame.possible_roles()), Step::Pass);
    }

    /// Lean `walk_sound`, on the whole walk for every chain of up to
    /// `FRAMES` known frames: the verdict admits the parser's scan over
    /// every concrete stack the chain describes. A concrete stack gives each
    /// known frame one of its possible namespaces. Below a truncated chain
    /// it has any ancestors at all, here up to `REVEALED` of them.
    #[kani::proof]
    #[kani::unwind(8)]
    fn the_walk_is_sound_for_every_stack_the_chain_describes() {
        let (frames, len) = any_frames::<FRAMES>();
        let truncated: bool = kani::any();

        let mut stack = [Role::Pass; REVEALED + FRAMES];
        let revealed: usize = kani::any();
        kani::assume(revealed <= REVEALED);
        kani::assume(truncated || revealed == 0);
        for slot in stack.iter_mut().take(revealed) {
            *slot = any_role();
        }
        for (index, frame) in frames.iter().enumerate().take(len) {
            let namespace: usize = kani::any();
            kani::assume(namespace < NAMESPACES && frame.has(namespace));
            stack[revealed + index] = frame.role(namespace);
        }

        let (verdict, decided_by) =
            walk(&frames[..len], exhausted(truncated), Frame::possible_roles);
        assert!(admits(verdict, run(&stack[..revealed + len])));
        // A proven scan names the frame that proves it, and only then.
        assert_eq!(decided_by.is_some(), verdict == Tri::Yes);
        if let Some(index) = decided_by {
            assert!(index < len);
            assert_eq!(
                classify(frames[index].possible_roles()),
                Step::Decided(Tri::Yes)
            );
        }
    }

    /// Lean `walk_stable`: a definite verdict on a truncated chain survives
    /// any ancestors the context later reveals, whatever lies below them.
    #[kani::proof]
    #[kani::unwind(8)]
    fn a_definite_truncated_verdict_survives_revealed_ancestors() {
        let (known, known_len) = any_frames::<FRAMES>();
        let (more, more_len) = any_frames::<REVEALED>();
        let (verdict, decided_by) = walk(&known[..known_len], Tri::Maybe, Frame::possible_roles);
        kani::assume(verdict != Tri::Maybe);

        // Frames are outermost first, so revealed ancestors go in front.
        let mut composed = [any_frame(); REVEALED + FRAMES];
        for (index, slot) in composed.iter_mut().enumerate().take(more_len + known_len) {
            *slot = if index < more_len {
                more[index]
            } else {
                known[index - more_len]
            };
        }
        let truncated: bool = kani::any();
        let (after, after_by) = walk(
            &composed[..more_len + known_len],
            exhausted(truncated),
            Frame::possible_roles,
        );
        assert_eq!(after, verdict);
        assert_eq!(after_by, decided_by.map(|index| index + more_len));
    }

    /// Lean `proven_sound`: a violation is proven when every possible parent
    /// dispatch diverges, and then it holds for whichever dispatch the page
    /// really takes. The checker folds the per-dispatch facts with
    /// `Tri::join`; a frame has at most four dispatches.
    #[kani::proof]
    #[kani::unwind(6)]
    fn a_proven_violation_holds_for_every_dispatch() {
        const DISPATCHES: usize = 4;
        let diverges = [any_tri(), any_tri(), any_tri(), any_tri()];
        let concrete: [bool; DISPATCHES] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= DISPATCHES);

        let mut outcome: Option<Tri> = None;
        for index in 0..len {
            kani::assume(admits(diverges[index], concrete[index]));
            outcome = Some(outcome.map_or(diverges[index], |so_far| so_far.join(diverges[index])));
        }
        let proven = outcome.unwrap_or(Tri::No) == Tri::Yes;

        let mut every_dispatch_diverges = len > 0;
        for index in 0..len {
            every_dispatch_diverges &= diverges[index] == Tri::Yes;
        }
        assert_eq!(proven, every_dispatch_diverges);
        if proven {
            for index in 0..len {
                assert!(concrete[index]);
            }
        }
    }
}
