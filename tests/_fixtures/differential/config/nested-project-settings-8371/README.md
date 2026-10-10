# Nested editor project settings (#8371)

The public JSON-RPC fixture creates two packages below one editor workspace.
Both import the same `@shared/contracts` alias, but each resolves it using a
package-relative custom tsconfig and exports a different type. Vite settings
also differ for hover and quotes. The fixture verifies concurrent requests,
initialization-option precedence, watched config replacement, and native
positive/negative diagnostic cases without creating a dedicated Vize config.

The source files are text fixtures so repository formatters do not normalize the
input before the public formatter request sees it.
