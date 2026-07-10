use vize_atlas::{
    Compilation, Product, ProductStatus, ProductView, Provider, ProviderContext, ProviderError,
    Shared,
};

struct SourceText;

impl Product for SourceText {
    type Value = Shared<str>;
    const NAME: &'static str = "test.source_text";
}

impl ProductView for SourceText {
    type View<'storage> = &'storage str;

    fn view(storage: &Shared<str>) -> &str {
        storage.as_ref()
    }
}

struct SourceTextProvider;

impl Provider for SourceTextProvider {
    type Product = SourceText;

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<Shared<str>, ProviderError> {
        Ok(context.source().shared_text())
    }
}

struct Words;

impl Product for Words {
    type Value = Shared<str>;
    const NAME: &'static str = "test.words";
}

impl ProductView for Words {
    type View<'storage> = std::str::SplitWhitespace<'storage>;

    fn view(storage: &Shared<str>) -> std::str::SplitWhitespace<'_> {
        storage.split_whitespace()
    }
}

struct WordsProvider;

impl Provider for WordsProvider {
    type Product = Words;

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<Shared<str>, ProviderError> {
        Ok(context.source().shared_text())
    }
}

fn compilation() -> Compilation {
    let mut compilation = Compilation::new();
    compilation.register_provider(SourceTextProvider).unwrap();
    compilation.register_provider(WordsProvider).unwrap();
    compilation
}

#[test]
fn borrowed_str_view_points_into_cached_static_storage() {
    let mut compilation = compilation();
    let source = compilation
        .add_source("component.tsx", "alpha beta")
        .unwrap();

    compilation.query::<SourceText>(source).unwrap();
    let cached = compilation.query::<SourceText>(source).unwrap();

    assert_eq!(cached.status(), ProductStatus::CacheHit);
    let view: &str = cached.view();
    assert_eq!(view, "alpha beta");
    assert_eq!(view.as_ptr(), cached.value().as_ptr());

    let execution_view = cached.execution().view::<SourceText>().unwrap().unwrap();
    assert_eq!(execution_view.as_ptr(), cached.value().as_ptr());
}

#[test]
fn iterator_view_streams_borrowed_items_from_cached_storage() {
    let mut compilation = compilation();
    let source = compilation
        .add_source("component.vue", "alpha beta gamma")
        .unwrap();

    compilation.query::<Words>(source).unwrap();
    let cached = compilation.query::<Words>(source).unwrap();

    assert_eq!(cached.status(), ProductStatus::CacheHit);
    let mut stream = cached.view();
    let first = stream.next().unwrap();
    assert_eq!(first.as_ptr(), cached.value().as_ptr());
    assert_eq!(
        std::iter::once(first).chain(stream).collect::<Vec<_>>(),
        ["alpha", "beta", "gamma"]
    );
}
