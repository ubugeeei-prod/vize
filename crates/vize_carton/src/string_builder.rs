//! String builder utilities for efficient string construction.
//!
//! Provides macros for building strings without excessive allocations,
//! avoiding the overhead of `format!` by using direct `push_str` operations.

/// Push multiple items to a string efficiently.
///
/// This macro expands to a series of `push_str` and `push` calls,
/// avoiding the allocation overhead of `format!`.
///
/// # Examples
///
/// ```
/// use vize_carton::appends;
///
/// let mut s = String::new();
/// appends!(s, "Hello, ", "world", "!");
/// assert_eq!(s, "Hello, world!");
/// ```
///
/// With numeric values (using write! internally):
///
/// ```
/// use vize_carton::appends;
///
/// let mut s = String::new();
/// let count = 42u32;
/// appends!(s, "Count: ", @count, " items");
/// assert_eq!(s, "Count: 42 items");
/// ```
///
/// With single characters:
///
/// ```
/// use vize_carton::appends;
///
/// let mut s = String::new();
/// appends!(s, "a", #':', "b");
/// assert_eq!(s, "a:b");
/// ```
#[macro_export]
macro_rules! appends {
    // Base case: no more items
    ($s:expr $(,)?) => {};

    // Single character literal (prefixed with #)
    ($s:expr, # $ch:literal $(, $($rest:tt)*)?) => {
        $s.push($ch);
        $crate::appends!($s $(, $($rest)*)?);
    };

    // Numeric value using write! (prefixed with @)
    ($s:expr, @ $num:expr $(, $($rest:tt)*)?) => {
        {
            use ::std::fmt::Write;
            let _ = write!($s, "{}", $num);
        }
        $crate::appends!($s $(, $($rest)*)?);
    };

    // String literal or &str (catch-all for expressions)
    ($s:expr, $item:expr $(, $($rest:tt)*)?) => {
        $s.push_str($item);
        $crate::appends!($s $(, $($rest)*)?);
    };
}

/// Push a line (with newline) to a string efficiently.
///
/// Equivalent to `appends!` followed by a newline character.
///
/// # Examples
///
/// ```
/// use vize_carton::appendln;
///
/// let mut s = String::new();
/// appendln!(s, "Hello, world!");
/// assert_eq!(s, "Hello, world!\n");
/// ```
#[macro_export]
macro_rules! appendln {
    ($s:expr $(, $($items:tt)*)?) => {
        $crate::appends!($s $(, $($items)*)?);
        $s.push('\n');
    };
}

/// Create a new [`CompactString`] using format arguments.
///
/// This avoids `format!()` (which returns `std::string::String`)
/// and directly builds a `CompactString`.
///
/// # Examples
///
/// ```
/// use vize_carton::cstr;
///
/// let name = "world";
/// let s = cstr!("Hello, {name}!");
/// assert_eq!(s, "Hello, world!");
/// ```
#[macro_export]
macro_rules! cstr {
    ($($arg:tt)*) => {{
        $crate::ToCompactString::to_compact_string(&::core::format_args!($($arg)*))
    }};
}

/// Push formatted content to a string using `write!`.
///
/// Works on both `std::string::String` and `CompactString` (anything
/// implementing `std::fmt::Write`).
///
/// # Examples
///
/// ```
/// use vize_carton::append;
///
/// let mut s = String::new();
/// let name = "world";
/// append!(s, "Hello, {name}!");
/// assert_eq!(s, "Hello, world!");
/// ```
#[macro_export]
macro_rules! append {
    ($dst:expr, $($arg:tt)*) => {{
        use ::std::fmt::Write as _;
        ::std::fmt::Write::write_fmt(&mut $dst, format_args!($($arg)*)).unwrap()
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_appends_strings() {
        let mut s = std::string::String::new();
        appends!(s, "Hello", ", ", "world", "!");
        assert_eq!(s, "Hello, world!");
    }

    #[test]
    fn test_appends_with_char() {
        let mut s = std::string::String::new();
        appends!(s, "a", #':', "b");
        assert_eq!(s, "a:b");
    }

    #[test]
    fn test_appends_with_number() {
        let mut s = std::string::String::new();
        let n = 42u32;
        appends!(s, "count: ", @n);
        assert_eq!(s, "count: 42");
    }

    #[test]
    fn test_appends_mixed() {
        let mut s = std::string::String::new();
        let start = 10usize;
        let end = 20usize;
        appends!(s, "range: ", @start, #':', @end);
        assert_eq!(s, "range: 10:20");
    }

    #[test]
    fn test_appendln() {
        let mut s = std::string::String::new();
        appendln!(s, "Hello");
        appendln!(s, "World");
        assert_eq!(s, "Hello\nWorld\n");
    }

    #[test]
    fn test_appends_empty() {
        let s = std::string::String::from("start");
        appends!(s);
        assert_eq!(s, "start");
    }
}
