//! Type-aware implementation / declaration / type-definition navigation from
//! authored Vue onto authored `.ts` and `.vue` sources through a real tsgo.
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "tests assert by panicking"
)]

use std::fs;
use std::sync::Arc;

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Url};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use super::{DeclarationService, IdeContext, ImplementationService, TypeDefinitionService};
use crate::server::ServerState;

const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#;

const SHAPES: &str = "export interface Shape {
  area(): number
}

export class Circle implements Shape {
  constructor(public r: number) {}
  area() {
    return 3 * this.r * this.r
  }
}

export class Square implements Shape {
  constructor(public s: number) {}
  area() {
    return this.s * this.s
  }
}

export type Point = { x: number; y: number }
";

const CARD: &str = r#"<script lang="ts">
import type { Shape as Plain } from './shapes'
export function measure(plain: Plain, scale: number): number {
  return plain.area() * scale
}
</script>

<script setup lang="ts">
import type { Shape, Point } from './shapes'
import { Circle } from './shapes'
const shape: Shape = new Circle(1)
const origin: Point = { x: 0, y: 0 }
function pick(point: Point): void {}
</script>

<template>
  <p>{{ shape.area() }} {{ origin.x }}</p>
  <p v-if="origin.y" :title="String(measure(shape, 2))" @click="pick(origin)">x</p>
</template>

<style>
/* v-bind(origin) */
.card { width: v-bind('measure(shape, 3) + "px"'); color: v-bind(origin.x); margin: v-bind('origin.y +') }
</style>
"#;

#[derive(Clone, Copy)]
enum Request {
    Implementation,
    Declaration,
    TypeDefinition,
}

struct Project {
    _root: tempfile::TempDir,
    shapes: Url,
    card: Url,
    state: ServerState,
    bridge: Arc<CorsaBridge>,
}

impl Project {
    async fn open(tsgo: std::path::PathBuf) -> Self {
        let root = crate::test_support::tempdir().expect("temp project");
        let src = root.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(root.path().join("tsconfig.json"), TSCONFIG).expect("tsconfig");
        fs::write(src.join("shapes.ts"), SHAPES).expect("shapes");
        fs::write(src.join("Card.vue"), CARD).expect("card");
        let shapes = Url::from_file_path(src.join("shapes.ts")).expect("shapes uri");
        let card = Url::from_file_path(src.join("Card.vue")).expect("card uri");
        let state = ServerState::new();
        state.set_workspace_root(root.path().to_path_buf());
        state
            .documents
            .open(card.clone(), CARD.to_owned(), 1, "vue".to_owned());
        state.update_virtual_docs(&card, CARD);
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo),
            working_dir: Some(root.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.expect("tsgo session");
        Self {
            _root: root,
            shapes,
            card,
            state,
            bridge,
        }
    }

    /// `(file name, authored text)` of every location for `request` at the
    /// `nth` byte of the last occurrence of `needle` in the card.
    async fn targets(&self, request: Request, needle: &str, nth: usize) -> Vec<(String, String)> {
        let offset = CARD.rfind(needle).expect("needle") + nth;
        let ctx = IdeContext::new(&self.state, &self.card, offset).expect("context");
        let bridge = Some(self.bridge.clone());
        let response = match request {
            Request::Implementation => {
                ImplementationService::implementation_with_corsa(&ctx, bridge).await
            }
            Request::Declaration => DeclarationService::declaration_with_corsa(&ctx, bridge).await,
            Request::TypeDefinition => {
                TypeDefinitionService::type_definition_with_corsa(&ctx, bridge).await
            }
        };
        let locations = match response {
            None => Vec::new(),
            Some(GotoDefinitionResponse::Scalar(location)) => vec![location],
            Some(GotoDefinitionResponse::Array(locations)) => locations,
            Some(GotoDefinitionResponse::Link(_)) => panic!("expected locations"),
        };
        let mut targets: Vec<_> = locations
            .iter()
            .map(|location| self.describe(location))
            .collect();
        targets.sort();
        targets
    }

    fn describe(&self, location: &Location) -> (String, String) {
        let (name, source) = if location.uri == self.shapes {
            ("shapes.ts", SHAPES)
        } else if location.uri == self.card {
            ("Card.vue", CARD)
        } else {
            panic!("unexpected target {}", location.uri);
        };
        let start = crate::ide::position_to_offset(
            source,
            location.range.start.line,
            location.range.start.character,
        )
        .expect("start");
        let end = crate::ide::position_to_offset(
            source,
            location.range.end.line,
            location.range.end.character,
        )
        .expect("end");
        (
            name.to_owned(),
            source.get(start..end).expect("range").to_owned(),
        )
    }

    async fn close(self) {
        self.bridge.shutdown().await.expect("shutdown");
    }
}

fn targets(entries: &[(&str, &str)]) -> Vec<(String, String)> {
    entries
        .iter()
        .map(|(file, text)| ((*file).to_owned(), (*text).to_owned()))
        .collect()
}

#[test]
fn navigation_from_vue_lands_on_authored_sources() {
    crate::runtime::block_on(async {
        let Some(tsgo) = crate::test_support::tsgo() else {
            return;
        };
        let project = Project::open(tsgo).await;
        let mut mismatches = Vec::new();
        // Like plain TypeScript, tsgo also lists the renamed specifier in
        // `import type { Shape as Plain }` as an implementation of `Shape`.
        for (request, needle, nth, expected) in [
            (
                Request::Implementation,
                "Shape, Point",
                2,
                vec![
                    ("Card.vue", "Shape"),
                    ("shapes.ts", "Circle"),
                    ("shapes.ts", "Square"),
                ],
            ),
            (
                Request::Implementation,
                "shape.area()",
                "shape.ar".len(),
                vec![("shapes.ts", "area"), ("shapes.ts", "area")],
            ),
            (
                Request::Declaration,
                "shape.area()",
                2,
                vec![("Card.vue", "shape")],
            ),
            (
                Request::Declaration,
                "origin.x }}",
                2,
                vec![("Card.vue", "origin")],
            ),
            (
                Request::TypeDefinition,
                "origin.x }}",
                2,
                vec![("shapes.ts", "{ x: number; y: number }")],
            ),
            (
                Request::TypeDefinition,
                "shape.area()",
                2,
                vec![("shapes.ts", "Shape")],
            ),
            (
                Request::Implementation,
                "plain.area()",
                "plain.ar".len(),
                vec![("shapes.ts", "area"), ("shapes.ts", "area")],
            ),
            (
                Request::Implementation,
                "Shape as Plain",
                1,
                vec![
                    ("Card.vue", "Shape"),
                    ("shapes.ts", "Circle"),
                    ("shapes.ts", "Square"),
                ],
            ),
            (
                Request::TypeDefinition,
                "plain.area()",
                2,
                vec![("shapes.ts", "Shape")],
            ),
            (
                Request::Declaration,
                "plain.area()",
                2,
                vec![("Card.vue", "plain")],
            ),
            (
                Request::Implementation,
                "Shape = new",
                1,
                vec![
                    ("Card.vue", "Shape"),
                    ("shapes.ts", "Circle"),
                    ("shapes.ts", "Square"),
                ],
            ),
            (
                Request::TypeDefinition,
                "origin: Point",
                1,
                vec![("shapes.ts", "{ x: number; y: number }")],
            ),
            (
                Request::TypeDefinition,
                "origin.y\"",
                2,
                vec![("shapes.ts", "{ x: number; y: number }")],
            ),
            (
                Request::Declaration,
                "measure(shape",
                2,
                vec![("Card.vue", "measure")],
            ),
            (
                Request::TypeDefinition,
                "shape, 2",
                1,
                vec![("shapes.ts", "Shape")],
            ),
            (
                Request::Implementation,
                "shape, 2",
                1,
                vec![("Card.vue", "shape")],
            ),
            (
                Request::Implementation,
                "shape.area()",
                2,
                vec![("Card.vue", "shape")],
            ),
            (
                Request::Declaration,
                "pick(origin)",
                1,
                vec![("Card.vue", "pick")],
            ),
            (
                Request::TypeDefinition,
                "origin)\"",
                2,
                vec![("shapes.ts", "{ x: number; y: number }")],
            ),
            (
                Request::Implementation,
                "pick(origin)",
                1,
                vec![("Card.vue", "pick")],
            ),
            (
                Request::TypeDefinition,
                "origin.x);",
                2,
                vec![("shapes.ts", "{ x: number; y: number }")],
            ),
            (
                Request::Implementation,
                "shape, 3)",
                1,
                vec![("Card.vue", "shape")],
            ),
            (
                Request::Declaration,
                "measure(shape, 3)",
                2,
                vec![("Card.vue", "measure")],
            ),
            (
                Request::TypeDefinition,
                "shape, 3)",
                1,
                vec![("shapes.ts", "Shape")],
            ),
            // An expression that does not parse, and a commented-out binding.
            (Request::Declaration, "origin.y +", 2, vec![]),
            (Request::Declaration, "origin) */", 2, vec![]),
        ] {
            let actual = project.targets(request, needle, nth).await;
            if actual != targets(&expected) {
                mismatches.push(vize_l0::cstr!("{needle} @{nth}: {actual:?}"));
            }
        }
        project.close().await;
        assert!(mismatches.is_empty(), "{mismatches:#?}");
    });
}
