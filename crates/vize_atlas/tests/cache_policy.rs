use std::sync::atomic::{AtomicUsize, Ordering};

use vize_atlas::{
    CachePolicy, Compilation, PlanningContext, Product, ProductId, ProductRequest, ProductStatus,
    ProductView, Provider, ProviderContext, ProviderError, Shared,
};
struct TransientWords;

impl Product for TransientWords {
    type Value = Shared<str>;

    const NAME: &'static str = "test.transient-words";
    const CACHE_POLICY: CachePolicy = CachePolicy::Transient;
}

impl ProductView for TransientWords {
    type View<'storage> = std::str::SplitWhitespace<'storage>;

    fn view(storage: &Self::Value) -> Self::View<'_> {
        storage.split_whitespace()
    }
}

struct WordsProvider(Shared<AtomicUsize>);

impl Provider for WordsProvider {
    type Product = TransientWords;

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<Shared<str>, ProviderError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(context.source().shared_text())
    }
}

struct Left;
struct Right;

impl Product for Left {
    type Value = usize;
    const NAME: &'static str = "test.left";
}

impl Product for Right {
    type Value = usize;
    const NAME: &'static str = "test.right";
}

struct CountProvider<P>(std::marker::PhantomData<P>);

impl<P> Provider for CountProvider<P>
where
    P: Product<Value = usize>,
{
    type Product = P;

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<TransientWords>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<usize, ProviderError> {
        Ok(context.get::<TransientWords>()?.split_whitespace().count())
    }
}

#[test]
fn transient_streams_share_within_a_plan_without_persistent_storage() {
    let calls = Shared::new(AtomicUsize::new(0));
    let mut compilation = Compilation::new();
    compilation
        .register_provider(WordsProvider(Shared::clone(&calls)))
        .unwrap();
    compilation
        .register_provider(CountProvider::<Left>(std::marker::PhantomData))
        .unwrap();
    compilation
        .register_provider(CountProvider::<Right>(std::marker::PhantomData))
        .unwrap();
    let source = compilation.add_source("words.txt", "typed stream").unwrap();

    let plan = compilation
        .plan(source, [ProductId::of::<Left>(), ProductId::of::<Right>()])
        .unwrap();
    let outcome = compilation.execute(plan).unwrap();
    let words = outcome.view::<TransientWords>().unwrap().unwrap();
    assert_eq!(words.collect::<Vec<_>>(), ["typed", "stream"]);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(!compilation.cache().contains::<TransientWords>(source));
    assert_eq!(compilation.cache().len(), 2);

    let cached_roots = compilation
        .plan(source, [ProductId::of::<Left>(), ProductId::of::<Right>()])
        .unwrap();
    let cached_outcome = compilation.execute(cached_roots).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        cached_outcome.status_for_request(ProductRequest::for_product::<TransientWords>(source)),
        Some(ProductStatus::Pruned)
    );

    compilation.query::<TransientWords>(source).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(!compilation.cache().contains::<TransientWords>(source));
}
