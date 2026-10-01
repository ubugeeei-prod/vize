# Caller-provided diagnostic message lookup

Issue: [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).

## Decision

L0 diagnostics accept a `diag::MessageLookup` provider whose only operation
returns `alloc::borrow::Cow<'static, str>` for a stable message key. The
provider contract contains no locale, catalog storage or global initialization.
Borrowed results name embedded text; owned results outlive a temporary key or
provider.

`ErrorCode::localized_message_with` and `localized_help_with` ask the supplied
provider for the existing `.message` and `.help` keys. The existing locale-based
methods bind the embedded translator's locale and delegate to these methods.
CLI explain pages and Relief diagnostics therefore consume the provider path
through their unchanged APIs. Native compiler calls to `ErrorCode::message`
continue to use the same static English text.

`Translator::for_locale` returns a borrowed `LocaleMessages` view of its tables.
`Translator::get` uses that view: requested-locale text wins, missing text falls
back only to English, and an unknown key becomes owned text. Formatting retains
the same ordered substitution loop, repeated replacements, unmatched
placeholders and empty-variable behavior. All catalog entries, loader behavior,
public locale values and existing localization signatures remain unchanged.

## Bounds and follow-up

This is the provider prerequisite for a later catalog ownership cut. Embedded
tables, supplemental entries, the loader and the global translator still live
in L0. Moving them to Carton requires a separate audit of actual lookup callers
and every normal/build dependency, including optional and target-specific
workspace paths. L0 must never depend on Carton. No pipeline stage,
serialization, dependency or performance ceiling is added here.

Tests cover explicit-provider keys and text, owned unknown-key lifetimes,
requested-locale selection, English fallback, substitution order and all 56
compiler codes in all three locales. Exact-head Actions and protected queue
validation remain required before merge. This slice does not complete #6834 or
replace any product's legacy compiler route.
