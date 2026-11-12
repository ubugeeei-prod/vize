//! Checker facts about TS type nodes: union constituents and empty object types.
use corsa::api::{ProjectSession, TypeHandle, TypeResponse};
use serde_json::Value;
use vize_l0::{FxHashMap, String as CompactString, cstr};

// TypeScript 7 TypeFlags / ObjectFlags, from microsoft/typescript-go
// internal/checker/types.go.
const ANY: u32 = 1;
const UNKNOWN: u32 = 1 << 1;
const BOOLEAN: u32 = 1 << 8;
const STRING_LITERAL: u32 = 1 << 10;
const NUMBER_LITERAL: u32 = 1 << 11;
const BIGINT_LITERAL: u32 = 1 << 12;
const BOOLEAN_LITERAL: u32 = 1 << 13;
const NEVER: u32 = 1 << 18;
const TEMPLATE_LITERAL: u32 = 1 << 22;
const OBJECT: u32 = 1 << 20;
const UNION: u32 = 1 << 27;
const CLASS_OR_INTERFACE: u32 = (1 << 0) | (1 << 1);

/// One union constituent of a type node's type.
#[derive(Clone)]
pub(in crate::linter) struct TypePartFact {
    /// Raw TypeScript 7 `TypeFlags`.
    pub flags: u32,
    /// How typescript-eslint names the constituent in messages
    /// (`describeLiteralType`): `"a"`, `1`, `-1n`, `true`, `any`, ...
    pub text: CompactString,
}

/// Intrinsic handles and per-type caches shared by one fact batch.
pub(super) struct TypeNodeProbe {
    any: Option<TypeHandle>,
    number: Option<TypeHandle>,
    string: Option<TypeHandle>,
    constituents: FxHashMap<TypeHandle, Vec<TypeResponse>>,
    empty: FxHashMap<TypeHandle, bool>,
}

impl TypeNodeProbe {
    /// Load the intrinsic types the requested facts compare against.
    pub(super) async fn new(
        session: &ProjectSession,
        parts: bool,
        empty_object: bool,
    ) -> corsa::Result<Self> {
        let client = session.client();
        let snapshot = || session.snapshot().handle.clone();
        let project = || session.project_handle();
        let any = if parts || empty_object {
            Some(client.get_any_type(snapshot(), project()).await?.id)
        } else {
            None
        };
        let (number, string) = if empty_object {
            (
                Some(client.get_number_type(snapshot(), project()).await?.id),
                Some(client.get_string_type(snapshot(), project()).await?.id),
            )
        } else {
            (None, None)
        };
        Ok(Self {
            any,
            number,
            string,
            constituents: FxHashMap::default(),
            empty: FxHashMap::default(),
        })
    }

    /// Whether `ty` is an `any`-flagged type other than the `any` intrinsic,
    /// which in a type position is the `error` type of an unresolved name.
    pub(super) fn is_error(&self, ty: &TypeResponse) -> bool {
        ty.flags == ANY && self.any.as_ref().is_some_and(|any| *any != ty.id)
    }

    /// typescript-eslint `unionTypePartsUnlessBoolean`: the union constituents
    /// of `ty`, keeping `boolean` (`false | true`) whole.
    pub(super) async fn parts(
        &mut self,
        session: &ProjectSession,
        ty: &TypeResponse,
    ) -> corsa::Result<Vec<TypePartFact>> {
        let parts = if ty.flags & BOOLEAN != 0 {
            vec![ty.clone()]
        } else {
            self.constituents(session, ty).await?
        };
        Ok(parts
            .iter()
            .map(|part| TypePartFact {
                flags: part.flags,
                text: describe(part),
            })
            .collect())
    }

    async fn constituents(
        &mut self,
        session: &ProjectSession,
        ty: &TypeResponse,
    ) -> corsa::Result<Vec<TypeResponse>> {
        if ty.flags & UNION == 0 {
            return Ok(vec![ty.clone()]);
        }
        if let Some(parts) = self.constituents.get(&ty.id) {
            return Ok(parts.clone());
        }
        let mut parts = session
            .client()
            .get_types_of_type_in_project(
                session.snapshot().handle.clone(),
                session.project_handle(),
                ty.id.clone(),
            )
            .await?;
        if parts.is_empty() {
            parts.push(ty.clone());
        }
        self.constituents.insert(ty.id.clone(), parts.clone());
        Ok(parts)
    }

    /// typescript-eslint `no-generated-empty-object-type`'s
    /// `containsEmptyObjectType`: `ty`, or one of its union constituents, is
    /// a member-less object type every primitive is assignable to.
    pub(super) async fn contains_empty_object(
        &mut self,
        session: &ProjectSession,
        ty: &TypeResponse,
    ) -> corsa::Result<bool> {
        if self.is_empty_object(session, ty).await? {
            return Ok(true);
        }
        if ty.flags & UNION == 0 {
            return Ok(false);
        }
        for part in self.constituents(session, ty).await? {
            if self.is_empty_object(session, &part).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn is_empty_object(
        &mut self,
        session: &ProjectSession,
        ty: &TypeResponse,
    ) -> corsa::Result<bool> {
        if ty.flags & OBJECT == 0 || ty.object_flags.unwrap_or(0) & CLASS_OR_INTERFACE != 0 {
            return Ok(false);
        }
        if let Some(empty) = self.empty.get(&ty.id) {
            return Ok(*empty);
        }
        let empty = self.has_no_members(session, &ty.id).await?
            && self.accepts_primitives(session, &ty.id).await?;
        self.empty.insert(ty.id.clone(), empty);
        Ok(empty)
    }

    async fn has_no_members(
        &self,
        session: &ProjectSession,
        id: &TypeHandle,
    ) -> corsa::Result<bool> {
        if !session.get_properties_of_type(id.clone()).await?.is_empty() {
            return Ok(false);
        }
        let infos = session
            .client()
            .get_index_infos_of_type(
                session.snapshot().handle.clone(),
                session.project_handle(),
                id.clone(),
            )
            .await?;
        if !infos.is_empty() {
            return Ok(false);
        }
        for kind in [0, 1] {
            if !session
                .get_signatures_of_type(id.clone(), kind)
                .await?
                .is_empty()
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Types still awaiting type arguments have no members either; probing
    /// with both `number` and `string` rules those out, as upstream does.
    async fn accepts_primitives(
        &self,
        session: &ProjectSession,
        id: &TypeHandle,
    ) -> corsa::Result<bool> {
        for primitive in [&self.number, &self.string] {
            let Some(primitive) = primitive else {
                return Ok(false);
            };
            let assignable = session
                .client()
                .is_type_assignable_to(
                    session.snapshot().handle.clone(),
                    session.project_handle(),
                    primitive.clone(),
                    id.clone(),
                )
                .await?;
            if !assignable {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// typescript-eslint `describeLiteralType`, from the flags and literal value.
fn describe(ty: &TypeResponse) -> CompactString {
    let value = ty.value.as_ref();
    let text = match ty.flags {
        // `Value`'s `Display` is JSON, so a string literal comes out quoted.
        STRING_LITERAL => value
            .filter(|value| value.is_string())
            .map(|value| cstr!("{value}")),
        NUMBER_LITERAL => value.and_then(Value::as_f64).map(number_text),
        BIGINT_LITERAL => value
            .and_then(Value::as_str)
            .map(|digits| cstr!("{digits}n")),
        BOOLEAN_LITERAL => value.and_then(Value::as_bool).map(|value| cstr!("{value}")),
        ANY => Some(CompactString::from("any")),
        NEVER => Some(CompactString::from("never")),
        UNKNOWN => Some(CompactString::from("unknown")),
        TEMPLATE_LITERAL => Some(CompactString::from("template literal type")),
        _ => None,
    };
    text.unwrap_or_else(|| CompactString::from("literal type"))
}

/// JavaScript `Number.prototype.toString` for the integral and finite cases.
fn number_text(value: f64) -> CompactString {
    if value.fract() == 0.0 && value.abs() < 1e21 {
        cstr!("{}", value as i64)
    } else {
        cstr!("{value}")
    }
}
