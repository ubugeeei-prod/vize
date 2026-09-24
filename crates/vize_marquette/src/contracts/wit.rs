//! Reading a contract surface from WIT sources (`wit` feature).
//!
//! The WIT is resolved by the Bytecode Alliance `wit-parser` — the parser
//! `wit-bindgen` and wasmtime use — so the surface describes exactly the
//! package guests compile against. The facts the handshake carries beside
//! the WIT (protocol version, page schemas, required features) come in as a
//! [`Protocol`] from the host that implements them.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

use vize_s0::{String, ToCompactString, cstr};
use wit_parser::{Handle, PackageId, Resolve, Type, TypeDefKind, TypeId, TypeOwner, WorldItem};

use super::{
    CONTRACT_SURFACE_FORMAT, CONTRACT_SURFACE_FORMAT_VERSION, Case, ContractSurface, Field,
    FunctionShape, InterfaceSurface, TypeShape, WorldSurface,
};

/// The contract facts the handshake carries beside the WIT.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Protocol {
    /// The integer protocol version the handshake negotiates.
    pub protocol_version: u32,
    /// Serialized payload schemas by page name.
    pub pages: BTreeMap<String, u32>,
    /// The features a guest of each world must offer, by world name.
    pub required_features: BTreeMap<String, BTreeSet<String>>,
}

/// Why WIT sources did not yield a surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitSurfaceError(pub String);

impl fmt::Display for WitSurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for WitSurfaceError {}

/// Resolve the WIT package in `dir` and describe it with `protocol`.
///
/// # Errors
///
/// `wit-parser`'s resolution error, a package without a version, or an invalid
/// reference in the resolved package.
pub fn surface_from_wit(
    dir: &Path,
    protocol: &Protocol,
) -> Result<ContractSurface, WitSurfaceError> {
    let mut resolve = Resolve::default();
    let (package_id, _sources) = resolve
        .push_dir(dir)
        .map_err(|error| WitSurfaceError(cstr!("{error:#}")))?;
    surface_from_resolve(&resolve, package_id, protocol)
}

fn unknown_id(kind: &str) -> WitSurfaceError {
    WitSurfaceError(cstr!("wit-parser returned an unknown {kind} id"))
}

fn surface_from_resolve(
    resolve: &Resolve,
    package_id: PackageId,
    protocol: &Protocol,
) -> Result<ContractSurface, WitSurfaceError> {
    let Some(package) = resolve.packages.get(package_id) else {
        return Err(unknown_id("package"));
    };
    let Some(version) = &package.name.version else {
        return Err(WitSurfaceError(cstr!(
            "the WIT package {} has no version",
            package.name
        )));
    };
    let render = Render(resolve);
    let interfaces = package
        .interfaces
        .iter()
        .map(|(name, &id)| {
            let interface = resolve
                .interfaces
                .get(id)
                .ok_or_else(|| unknown_id("interface"))?;
            let mut types = BTreeMap::new();
            for (name, &ty) in &interface.types {
                if let Some(shape) = render.shape(ty)? {
                    types.insert(name.to_compact_string(), shape);
                }
            }
            let functions = interface
                .functions
                .iter()
                .map(|(name, function)| {
                    let shape = FunctionShape {
                        params: function
                            .params
                            .iter()
                            .map(|param| {
                                Ok(Field {
                                    name: param.name.to_compact_string(),
                                    ty: render.ty(&param.ty)?,
                                })
                            })
                            .collect::<Result<Vec<_>, WitSurfaceError>>()?,
                        result: function
                            .result
                            .as_ref()
                            .map(|ty| render.ty(ty))
                            .transpose()?,
                    };
                    Ok((name.to_compact_string(), shape))
                })
                .collect::<Result<BTreeMap<_, _>, WitSurfaceError>>()?;
            Ok((
                name.to_compact_string(),
                InterfaceSurface { types, functions },
            ))
        })
        .collect::<Result<BTreeMap<_, _>, WitSurfaceError>>()?;
    let worlds = package
        .worlds
        .iter()
        .map(|(name, &id)| {
            let world = resolve.worlds.get(id).ok_or_else(|| unknown_id("world"))?;
            let items = |items: &wit_parser::IndexMap<_, WorldItem>| -> Result<BTreeSet<String>, WitSurfaceError> {
                let mut names = BTreeSet::new();
                for (key, item) in items {
                    match item {
                        WorldItem::Interface { id, .. } => {
                            names.insert(render.interface(*id)?);
                        }
                        WorldItem::Function(function) => {
                            for param in &function.params {
                                render.ty(&param.ty)?;
                            }
                            if let Some(result) = &function.result {
                                render.ty(result)?;
                            }
                            names.insert(cstr!("func:{}", resolve.name_world_key(key)));
                        }
                        // World types do not have a surface entry, but their
                        // references still must resolve before accepting it.
                        WorldItem::Type { id, .. } => {
                            render.shape(*id)?;
                        }
                    }
                }
                Ok(names)
            };
            let name = name.to_compact_string();
            let surface = WorldSurface {
                imports: items(&world.imports)?,
                exports: items(&world.exports)?,
                required_features: protocol
                    .required_features
                    .get(&name)
                    .cloned()
                    .unwrap_or_default(),
            };
            Ok((name, surface))
        })
        .collect::<Result<BTreeMap<_, _>, WitSurfaceError>>()?;
    Ok(ContractSurface {
        format: String::from(CONTRACT_SURFACE_FORMAT),
        format_version: CONTRACT_SURFACE_FORMAT_VERSION,
        package: cstr!("{}:{}", package.name.namespace, package.name.name),
        version: version.to_compact_string(),
        protocol_version: protocol.protocol_version,
        pages: protocol.pages.clone(),
        interfaces,
        worlds,
    })
}

/// WIT spellings of types, named types qualified by their owner.
struct Render<'a>(&'a Resolve);

impl Render<'_> {
    fn interface(&self, id: wit_parser::InterfaceId) -> Result<String, WitSurfaceError> {
        let interface = self
            .0
            .interfaces
            .get(id)
            .ok_or_else(|| unknown_id("interface"))?;
        Ok(match interface.name.as_ref() {
            Some(name) => name.to_compact_string(),
            None => self.0.id_of(id).unwrap_or_default().to_compact_string(),
        })
    }

    /// The named shape of `id`, or `None` for a transparent alias.
    fn shape(&self, id: TypeId) -> Result<Option<TypeShape>, WitSurfaceError> {
        let fields = |fields: &[wit_parser::Field]| -> Result<Vec<Field>, WitSurfaceError> {
            fields
                .iter()
                .map(|field| {
                    Ok(Field {
                        name: field.name.to_compact_string(),
                        ty: self.ty(&field.ty)?,
                    })
                })
                .collect()
        };
        let def = self.0.types.get(id).ok_or_else(|| unknown_id("type"))?;
        Ok(Some(match &def.kind {
            TypeDefKind::Type(inner) => {
                self.ty(inner)?;
                return Ok(None);
            }
            TypeDefKind::Record(record) => TypeShape::Record(fields(&record.fields)?),
            TypeDefKind::Variant(variant) => TypeShape::Variant(
                variant
                    .cases
                    .iter()
                    .map(|case| {
                        Ok(Case {
                            name: case.name.to_compact_string(),
                            ty: case.ty.as_ref().map(|ty| self.ty(ty)).transpose()?,
                        })
                    })
                    .collect::<Result<Vec<_>, WitSurfaceError>>()?,
            ),
            TypeDefKind::Enum(enum_) => TypeShape::Enum(
                enum_
                    .cases
                    .iter()
                    .map(|case| case.name.to_compact_string())
                    .collect(),
            ),
            TypeDefKind::Flags(flags) => TypeShape::Flags(
                flags
                    .flags
                    .iter()
                    .map(|flag| flag.name.to_compact_string())
                    .collect(),
            ),
            TypeDefKind::Resource => TypeShape::Resource,
            kind => TypeShape::Alias(self.kind(kind)?),
        }))
    }

    fn ty(&self, ty: &Type) -> Result<String, WitSurfaceError> {
        Ok(String::from(match ty {
            Type::Bool => "bool",
            Type::U8 => "u8",
            Type::U16 => "u16",
            Type::U32 => "u32",
            Type::U64 => "u64",
            Type::S8 => "s8",
            Type::S16 => "s16",
            Type::S32 => "s32",
            Type::S64 => "s64",
            Type::F32 => "f32",
            Type::F64 => "f64",
            Type::Char => "char",
            Type::String => "string",
            Type::ErrorContext => "error-context",
            Type::Id(id) => return self.id(*id),
        }))
    }

    fn id(&self, id: TypeId) -> Result<String, WitSurfaceError> {
        let def = self.0.types.get(id).ok_or_else(|| unknown_id("type"))?;
        match (&def.name, &def.kind) {
            (_, TypeDefKind::Type(inner)) => self.ty(inner),
            (Some(name), _) => match def.owner {
                TypeOwner::Interface(owner) => Ok(cstr!("{}.{name}", self.interface(owner)?)),
                TypeOwner::World(world) => {
                    let world = self
                        .0
                        .worlds
                        .get(world)
                        .ok_or_else(|| unknown_id("world"))?;
                    Ok(cstr!("{}.{name}", world.name))
                }
                TypeOwner::None => Ok(name.to_compact_string()),
            },
            (None, kind) => self.kind(kind),
        }
    }

    fn kind(&self, kind: &TypeDefKind) -> Result<String, WitSurfaceError> {
        let opt = |ty: &Option<Type>| -> Result<String, WitSurfaceError> {
            ty.as_ref().map_or(Ok(String::from("_")), |ty| self.ty(ty))
        };
        Ok(match kind {
            TypeDefKind::List(ty) => cstr!("list<{}>", self.ty(ty)?),
            TypeDefKind::FixedLengthList(ty, len) => cstr!("list<{}, {len}>", self.ty(ty)?),
            TypeDefKind::Option(ty) => cstr!("option<{}>", self.ty(ty)?),
            TypeDefKind::Map(key, value) => cstr!("map<{}, {}>", self.ty(key)?, self.ty(value)?),
            TypeDefKind::Result(result) => {
                cstr!("result<{}, {}>", opt(&result.ok)?, opt(&result.err)?)
            }
            TypeDefKind::Tuple(tuple) => {
                let types: Vec<String> = tuple
                    .types
                    .iter()
                    .map(|ty| self.ty(ty))
                    .collect::<Result<_, _>>()?;
                cstr!("tuple<{}>", types.join(", "))
            }
            TypeDefKind::Handle(Handle::Own(id)) => cstr!("own<{}>", self.id(*id)?),
            TypeDefKind::Handle(Handle::Borrow(id)) => cstr!("borrow<{}>", self.id(*id)?),
            TypeDefKind::Future(ty) => cstr!("future<{}>", opt(ty)?),
            TypeDefKind::Stream(ty) => cstr!("stream<{}>", opt(ty)?),
            TypeDefKind::Type(ty) => return self.ty(ty),
            TypeDefKind::Record(_)
            | TypeDefKind::Variant(_)
            | TypeDefKind::Enum(_)
            | TypeDefKind::Flags(_)
            | TypeDefKind::Resource
            | TypeDefKind::Unknown => String::from("unnamed"),
        })
    }
}

#[cfg(test)]
mod tests;
