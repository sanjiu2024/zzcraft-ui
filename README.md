# zzcraft-ui

Missing pieces for building information-dense [GPUI](https://gpui.rs) applications
on top of [GPUI Kit](https://gpui-kit.com).

GPUI Kit ships the primitives — tables, dialogs, date pickers, charts. This crate
ships what a management console still needs on its own:

| module | what it adds |
| --- | --- |
| `format` | money and count formatting with locale separators, compact notation, and a defined value for broken numbers |
| `table` | table shell: column visibility, page size with totals, row action menu, cross-page selection |
| `form` | validation layer: sync and async rules, error aggregation, validate-after-first-submit |

## Usage

```toml
[dependencies]
zzcraft-ui = "0.1"
```

The `format` module has no UI dependency:

```rust
use zzcraft_ui::format::{Currency, format_money, format_compact};

assert_eq!(format_money(1234.5, Currency::USD), "$1,234.50");
assert_eq!(format_compact(1_200_000.0), "1.2M");
```

The rendering modules need the `gpui` feature:

```toml
zzcraft-ui = { version = "0.1", features = ["gpui"] }
```

## Why it exists

Every screen of a console shows the same three things: a number that must not be
formatted wrong, a table that must not become unusable once it has a page size
and a row menu, and a form whose validation must not be reinvented per screen.
Those are the parts this crate owns.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
