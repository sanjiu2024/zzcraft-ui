//! Number and money formatting.
//!
//! GPUI Kit's i18n only translates component strings: its documentation states
//! that date, number and currency formatting stay with the application. Every
//! screen that shows a balance, a price or a token count needs the same rules,
//! so they live here instead of being copied per screen.
//!
//! The rules that matter in practice:
//!
//! - money keeps two decimals and never switches to scientific notation
//! - counts drop the decimal tail but keep grouping separators
//! - large counts fall back to compact notation (`1.2M`), which is what an
//!   information-dense table has room for
//! - the decimal separator follows the locale, because a price rendered with the
//!   wrong separator is a wrong price to the person reading it

/// Currency whose symbol and precision are known to this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    CNY,
    USD,
}

impl Currency {
    /// Symbol shown in front of the amount.
    pub fn symbol(self) -> &'static str {
        match self {
            Currency::CNY => "¥",
            Currency::USD => "$",
        }
    }

    /// Digits kept after the decimal separator.
    ///
    /// Both currencies settle in cents, so two decimals is the exact answer, not
    /// a display choice.
    pub fn decimals(self) -> usize {
        2
    }
}

/// How the digits are laid out.
#[derive(Debug, Clone, Copy)]
pub struct FormatOptions {
    /// Digits after the decimal separator.
    pub decimals: usize,
    /// Insert a separator every three integer digits.
    pub grouping: bool,
    /// Decimal separator to use, `.` or `,`.
    pub decimal_separator: char,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            decimals: 0,
            grouping: true,
            decimal_separator: '.',
        }
    }
}

/// Format an amount of money with its symbol.
///
/// Negative amounts keep their sign in front of the symbol (`-$1.20`), which is
/// how an invoice reads; a value that is not finite renders as `—` so a broken
/// number never looks like a real price.
pub fn format_money(value: f64, currency: Currency) -> String {
    if !value.is_finite() {
        return "—".to_string();
    }
    let options = FormatOptions {
        decimals: currency.decimals(),
        ..FormatOptions::default()
    };
    let sign = if value < 0.0 { "-" } else { "" };
    format!(
        "{sign}{}{}",
        currency.symbol(),
        format_with(value.abs(), options)
    )
}

/// Format a plain number with the given options.
pub fn format_number(value: f64, options: FormatOptions) -> String {
    if !value.is_finite() {
        return "—".to_string();
    }
    format_with(value, options)
}

/// Format a count in compact notation (`852`, `12.3K`, `1.2M`).
///
/// Counts in a table are read to compare orders of magnitude, so the tail is
/// dropped once it stops being informative.
pub fn format_compact(value: f64) -> String {
    if !value.is_finite() {
        return "—".to_string();
    }
    let sign = if value < 0.0 { "-" } else { "" };
    let magnitude = value.abs();
    let units = [(1e12, "T"), (1e9, "B"), (1e6, "M"), (1e3, "K")];
    for (threshold, suffix) in units {
        if magnitude >= threshold {
            let scaled = magnitude / threshold;
            // One decimal below ten (1.2M), none above it (12M).
            let text = if scaled >= 10.0 {
                format!("{scaled:.0}")
            } else {
                format!("{scaled:.1}")
            };
            return format!("{sign}{text}{suffix}");
        }
    }
    format!("{sign}{magnitude:.0}")
}

fn format_with(value: f64, options: FormatOptions) -> String {
    let sign = if value < 0.0 { "-" } else { "" };
    let magnitude = value.abs();
    let fixed = format!("{magnitude:.*}", options.decimals);

    let (integer, fraction) = match fixed.split_once('.') {
        Some((integer, fraction)) => (integer.to_string(), Some(fraction.to_string())),
        None => (fixed, None),
    };

    let integer = if options.grouping {
        group_digits(&integer, group_separator(options.decimal_separator))
    } else {
        integer
    };

    match fraction {
        Some(fraction) => format!("{sign}{integer}{}{fraction}", options.decimal_separator),
        None => format!("{sign}{integer}"),
    }
}

// The grouping separator is the other one of the two conventions: a locale that
// writes 1,5 never also writes 1.234,567.
fn group_separator(decimal_separator: char) -> char {
    if decimal_separator == ',' { '.' } else { ',' }
}

fn group_digits(digits: &str, separator: char) -> String {
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(separator);
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_keeps_two_decimals_and_groups_thousands() {
        assert_eq!(format_money(1234.5, Currency::USD), "$1,234.50");
        assert_eq!(format_money(0.0, Currency::CNY), "¥0.00");
        assert_eq!(format_money(1234567.891, Currency::CNY), "¥1,234,567.89");
    }

    #[test]
    fn money_puts_the_sign_before_the_symbol() {
        assert_eq!(format_money(-12.3, Currency::USD), "-$12.30");
    }

    #[test]
    fn money_renders_unusable_values_as_a_dash() {
        assert_eq!(format_money(f64::NAN, Currency::USD), "—");
        assert_eq!(format_money(f64::INFINITY, Currency::USD), "—");
    }

    #[test]
    fn numbers_group_and_respect_locale_separator() {
        let grouped = FormatOptions::default();
        assert_eq!(format_number(1234567.0, grouped), "1,234,567");

        let comma_decimal = FormatOptions {
            decimals: 1,
            grouping: true,
            decimal_separator: ',',
        };
        assert_eq!(format_number(1234.5, comma_decimal), "1.234,5");
    }

    #[test]
    fn compact_notation_switches_unit_at_the_threshold() {
        assert_eq!(format_compact(852.0), "852");
        assert_eq!(format_compact(999.0), "999");
        assert_eq!(format_compact(1000.0), "1.0K");
        assert_eq!(format_compact(12_300.0), "12K");
        assert_eq!(format_compact(1_200_000.0), "1.2M");
        assert_eq!(format_compact(-1_500_000_000.0), "-1.5B");
    }

    #[test]
    fn compact_notation_handles_zero_and_broken_values() {
        assert_eq!(format_compact(0.0), "0");
        assert_eq!(format_compact(f64::NAN), "—");
    }
}
