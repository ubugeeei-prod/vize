use alloc::borrow::Cow;
use core::cell::RefCell;

use crate::diag::MessageLookup;
use crate::i18n::{Locale, translator};

use super::ErrorCode;

#[cfg(test)]
struct TestMessages {
    requests: RefCell<std::vec::Vec<std::string::String>>,
}

impl MessageLookup for TestMessages {
    fn lookup(&self, key: &str) -> Cow<'static, str> {
        self.requests.borrow_mut().push(key.to_owned());
        match key {
            "compiler/duplicate-attribute.message" => Cow::Borrowed("custom headline"),
            "compiler/duplicate-attribute.help" => Cow::Borrowed("custom remedy"),
            _ => Cow::Owned(key.to_owned()),
        }
    }
}

#[test]
fn localization_uses_only_the_supplied_provider_and_stable_keys() {
    let messages = TestMessages {
        requests: RefCell::default(),
    };
    let lookup: &dyn MessageLookup = &messages;
    let code = ErrorCode::DuplicateAttribute;
    assert_eq!(code.localized_message_with(lookup), "custom headline");
    assert_eq!(code.localized_help_with(lookup), "custom remedy");
    assert_eq!(
        *messages.requests.borrow(),
        [
            "compiler/duplicate-attribute.message",
            "compiler/duplicate-attribute.help",
        ]
    );
}

#[test]
fn unknown_provider_keys_remain_owned_after_the_provider_is_dropped() {
    let (message, help) = {
        let messages = TestMessages {
            requests: RefCell::default(),
        };
        let code = ErrorCode::MissingEndTag;
        (
            code.localized_message_with(&messages),
            code.localized_help_with(&messages),
        )
    };
    assert_eq!(message, "compiler/missing-end-tag.message");
    assert_eq!(help, "compiler/missing-end-tag.help");
}

#[test]
fn all_existing_code_and_locale_wrappers_keep_the_catalog_text() {
    let translator = translator();
    for &locale in Locale::ALL {
        let messages = translator.for_locale(locale);
        for code in ErrorCode::ALL {
            let message_key = crate::cstr!("{}.message", code.code());
            let help_key = crate::cstr!("{}.help", code.code());
            let message = translator.get(locale, &message_key);
            let help = translator.get(locale, &help_key);
            assert_eq!(code.localized_message_with(&messages).as_str(), message);
            assert_eq!(code.localized_help_with(&messages).as_str(), help);
            assert_eq!(code.localized_message(locale).as_str(), message);
            assert_eq!(code.localized_help(locale).as_str(), help);
        }
    }
}
