//! TS-16 laws for the fusion-plan page (`[fusion-plan-folio]`, C-3): the
//! page of a pipeline prints exactly, canonical text is a print/parse fixed
//! point, malformed records are refused with their line - and the walks it
//! names are the walks the pass manager actually runs, observed through the
//! run's own events rather than through the const grouping the page reads.

#![expect(clippy::expect_used, reason = "tests assert by panicking")]

use vize_davinci::folio::plan::{FolioPlanPass, FusionPlanFolio};
use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_davinci::pass::{
    BudgetObserver, Fusability, PassDesc, PassEvent, PassKind, PassObserver, Pipeline, Preserved,
    run_pipeline,
};
use vize_s0::{String, cstr};

const fn fusable(name: &'static str) -> PassDesc {
    PassDesc::new(
        name,
        PassKind::Optional,
        Fusability::Fusable,
        Preserved::ALL,
    )
}

const CHECK: PassDesc = PassDesc::new(
    "check",
    PassKind::MandatoryDiagnostic,
    Fusability::Barrier,
    Preserved::ALL,
);
const PASSES: &[PassDesc] = &[
    fusable("normalize"),
    fusable("fold"),
    CHECK,
    fusable("tidy"),
    fusable("sweep"),
];
const PLAN: Pipeline = Pipeline::new("s2", PASSES);

/// Two fusable runs around a barrier: three walks for five passes.
const CANONICAL: &str = "\
[fusion-plan-folio]
stage=s2
walks=3

[fusion-plan-folio.passes]
walk=0 pass=normalize kind=optional fusability=fusable
walk=0 pass=fold kind=optional fusability=fusable
walk=1 pass=check kind=mandatory-diagnostic fusability=barrier
walk=2 pass=tidy kind=optional fusability=fusable
walk=2 pass=sweep kind=optional fusability=fusable

";

#[test]
fn the_plan_of_a_pipeline_prints_exactly() {
    let folio = FusionPlanFolio::of(&PLAN);
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    // A derived page prints the same canonical text in both modes.
    assert_eq!(
        folio.print_to_string(FolioMode::Display),
        folio.print_to_string(FolioMode::Full)
    );
}

/// Records `(walk, pass)` from the events the pass manager raises.
#[derive(Default)]
struct WalkLog {
    passes: Vec<(u32, &'static str)>,
}

impl PassObserver for WalkLog {
    fn before_pass(&mut self, event: &PassEvent<'_>) {
        let walk = u32::try_from(event.group_index).expect("small pipeline");
        self.passes.push((walk, event.desc().name));
    }
}

#[test]
fn the_page_names_the_walks_the_pass_manager_runs() {
    let mut log = WalkLog::default();
    run_pipeline(&PLAN, &mut log, |_| Ok(())).expect("no-op bodies cannot fail");
    let mut budget = BudgetObserver::new();
    run_pipeline(&PLAN, &mut budget, |_| Ok(())).expect("no-op bodies cannot fail");

    let folio = FusionPlanFolio::of(&PLAN);
    let paged: Vec<(u32, &str)> = folio
        .passes
        .iter()
        .map(|pass| (pass.walk, pass.pass.as_str()))
        .collect();
    let observed: Vec<(u32, &str)> = log
        .passes
        .iter()
        .map(|&(walk, name)| (walk, name))
        .collect();
    assert_eq!(paged, observed);
    assert_eq!(folio.walks, budget.walks);
    assert_eq!(budget.passes, 5);
}

#[test]
fn an_empty_plan_costs_no_walk() {
    let folio = FusionPlanFolio::of(&Pipeline::new("s2", &[]));
    assert_eq!(
        folio.print_to_string(FolioMode::Full).as_str(),
        "[fusion-plan-folio]\nstage=s2\nwalks=0\n\n"
    );
    assert_eq!(
        FusionPlanFolio::parse("[fusion-plan-folio]\nstage=s2\nwalks=0\n\n"),
        Ok(folio)
    );
}

#[test]
fn full_print_is_identity_on_canonical_text_and_values_round_trip() {
    let parsed = FusionPlanFolio::parse(CANONICAL).expect("canonical text parses");
    assert_eq!(parsed.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    assert_eq!(parsed, FusionPlanFolio::of(&PLAN));

    let value = FusionPlanFolio {
        stage: String::from("s2-to-s3"),
        walks: 1,
        passes: vec![FolioPlanPass {
            walk: 0,
            pass: String::from("lower"),
            kind: PassKind::MandatoryLowering,
            fusability: Fusability::Barrier,
        }],
    };
    let printed = value.print_to_string(FolioMode::Full);
    assert_eq!(FusionPlanFolio::parse(printed.as_str()), Ok(value));
}

#[test]
fn malformed_records_are_refused_with_their_line() {
    let page = |record: &str| {
        cstr!("[fusion-plan-folio]\nstage=s2\nwalks=1\n\n[fusion-plan-folio.passes]\n{record}\n\n")
    };
    let refused = |record: &str| FusionPlanFolio::parse(page(record).as_str()).unwrap_err();
    assert_eq!(
        refused("walk=x pass=a kind=optional fusability=fusable"),
        FolioError::new(6, cstr!("invalid `walk` integer `x`"))
    );
    assert_eq!(
        refused("walk=0 pass= kind=optional fusability=fusable"),
        FolioError::new(6, cstr!("empty pass name"))
    );
    assert_eq!(
        refused("walk=0 pass=a kind=fast fusability=fusable"),
        FolioError::new(6, cstr!("unknown pass kind `fast`"))
    );
    assert_eq!(
        refused("walk=0 pass=a kind=optional fusability=fused"),
        FolioError::new(6, cstr!("unknown fusability `fused`"))
    );
    assert_eq!(
        refused("walk=0 pass=a kind=optional"),
        FolioError::new(6, cstr!("missing `fusability` field"))
    );
    assert_eq!(
        refused("walk=0 name=a kind=optional fusability=fusable"),
        FolioError::new(6, cstr!("expected `pass=...`, got `name=a`"))
    );
    assert_eq!(
        refused("walk=0 pass=a kind=optional fusability=fusable x=1"),
        FolioError::new(6, cstr!("unexpected field `x=1`"))
    );
}
