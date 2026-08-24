use vize_atelier_sfc::{
    BlockLocation, PadOption, SfcCustomBlock, SfcDescriptor, SfcError, SfcParseOptions,
    SfcScriptBlock, SfcStyleBlock, SfcTemplateBlock, parse_sfc,
};
use vize_croquis::sfc::SfcDescriptor as CroquisSfcDescriptor;

#[test]
fn full_compiler_reexports_the_parse_only_descriptor_type() {
    let descriptor: CroquisSfcDescriptor<'_> =
        parse_sfc("<template><div /></template>", SfcParseOptions::default()).unwrap();

    assert!(descriptor.template.is_some());
}

#[test]
fn full_compiler_preserves_every_legacy_parse_import_path() {
    use vize_atelier_sfc::parse::parse_sfc as parse_from_module;
    use vize_atelier_sfc::types::{
        BlockLocation as TypesBlockLocation, PadOption as TypesPadOption,
        SfcCustomBlock as TypesSfcCustomBlock, SfcDescriptor as TypesSfcDescriptor,
        SfcError as TypesSfcError, SfcParseOptions as TypesSfcParseOptions,
        SfcScriptBlock as TypesSfcScriptBlock, SfcStyleBlock as TypesSfcStyleBlock,
        SfcTemplateBlock as TypesSfcTemplateBlock,
    };

    let descriptor: TypesSfcDescriptor<'_> = parse_from_module(
        "<template><div /></template>",
        TypesSfcParseOptions::default(),
    )
    .unwrap();
    assert!(descriptor.template.is_some());

    let _root_paths = (
        std::any::TypeId::of::<BlockLocation>(),
        std::any::TypeId::of::<PadOption>(),
        std::any::TypeId::of::<SfcCustomBlock<'static>>(),
        std::any::TypeId::of::<SfcDescriptor<'static>>(),
        std::any::TypeId::of::<SfcError>(),
        std::any::TypeId::of::<SfcParseOptions>(),
        std::any::TypeId::of::<SfcScriptBlock<'static>>(),
        std::any::TypeId::of::<SfcStyleBlock<'static>>(),
        std::any::TypeId::of::<SfcTemplateBlock<'static>>(),
    );
    let _types_paths = (
        std::any::TypeId::of::<TypesBlockLocation>(),
        std::any::TypeId::of::<TypesPadOption>(),
        std::any::TypeId::of::<TypesSfcCustomBlock<'static>>(),
        std::any::TypeId::of::<TypesSfcError>(),
        std::any::TypeId::of::<TypesSfcScriptBlock<'static>>(),
        std::any::TypeId::of::<TypesSfcStyleBlock<'static>>(),
        std::any::TypeId::of::<TypesSfcTemplateBlock<'static>>(),
    );
}
