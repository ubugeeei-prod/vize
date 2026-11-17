//! Whether an unbound-method receiver is the global object.
//!
//! Platform operations of the global object (`setTimeout`, `matchMedia`,
//! `requestAnimationFrame`) use the global when they are called without a
//! receiver (WebIDL), so reading one off `window` does not lose `this`.
use super::{INTERSECTION, JsonMap, Shapes, UNION};
use corsa::api::TypeResponse;
use serde_json::Value;

// TypeScript 7 TypeFlags.Undefined and TypeFlags.Null.
const UNDEFINED: u32 = 1 << 2;
const NULL: u32 = 1 << 3;
/// The name of the symbol of `typeof globalThis`.
const GLOBAL_THIS: &str = "globalThis";

impl Shapes<'_> {
    /// The type is the global object: `typeof globalThis`, alone or in an
    /// intersection (`Window & typeof globalThis`, `WindowProxy & typeof
    /// globalThis`), optionally in a union with `null` or `undefined`
    /// (`document.defaultView`). A receiver typed `Window` alone (`parent`,
    /// another frame) is not. `receiver` holds the facts already read for
    /// the type's own symbol.
    pub(super) async fn is_global_object(&self, ty: &TypeResponse, receiver: &JsonMap) -> bool {
        if ty.flags & (UNION | INTERSECTION) == 0 {
            return receiver.get("receiverSymbol").and_then(Value::as_str) == Some(GLOBAL_THIS);
        }
        let unions = if ty.flags & UNION != 0 {
            self.session.constituents(ty).await
        } else {
            vec![ty.clone()]
        };
        let mut any = false;
        for part in unions {
            if part.flags & (NULL | UNDEFINED) != 0 {
                continue;
            }
            let pieces = if part.flags & INTERSECTION != 0 {
                self.session.constituents(&part).await
            } else {
                vec![part]
            };
            let mut global = false;
            for piece in pieces.iter().filter(|piece| piece.symbol.is_some()) {
                if self.is_global_this(piece).await {
                    global = true;
                    break;
                }
            }
            if !global {
                return false;
            }
            any = true;
        }
        any
    }

    async fn is_global_this(&self, ty: &TypeResponse) -> bool {
        let symbol = self
            .session
            .client()
            .get_symbol_of_type_in_project(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await;
        matches!(symbol, Ok(Some(symbol)) if symbol.name == GLOBAL_THIS)
    }
}
