use super::{Locale, translator};
use vize_l0::compiler_error::ErrorCode;

#[test]
fn all_existing_code_and_locale_wrappers_keep_the_catalog_text() {
    let translator = translator();
    for &locale in Locale::ALL {
        let messages = translator.for_locale(locale);
        for code in ErrorCode::ALL {
            let message_key = vize_l0::cstr!("{}.message", code.code());
            let help_key = vize_l0::cstr!("{}.help", code.code());
            let message = translator.get(locale, &message_key);
            let help = translator.get(locale, &help_key);
            assert_eq!(code.localized_message_with(&messages).as_str(), message);
            assert_eq!(code.localized_help_with(&messages).as_str(), help);
        }
    }
}
