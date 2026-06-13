use vize_carton::CompactString;
use vize_croquis::provide::ProvideKey;

#[test]
fn test_provide_key_match() {
    let key1 = ProvideKey::String(CompactString::new("theme"));
    let key2 = ProvideKey::String(CompactString::new("theme"));

    let s1 = match &key1 {
        ProvideKey::String(s) => s.as_str(),
        ProvideKey::Symbol(s) => s.as_str(),
    };
    let s2 = match &key2 {
        ProvideKey::String(s) => s.as_str(),
        ProvideKey::Symbol(s) => s.as_str(),
    };

    assert_eq!(s1, s2);
}
