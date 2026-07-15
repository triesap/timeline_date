# timeline_date

Localised timeline date labels for Rust applications.

## Install

```toml
[dependencies]
timeline_date = "0.1.0"
```

## Purpose

- Format feed, detail, and audit labels from explicit event and current timestamps.
- Classify dates with user-timezone civil-day semantics, not UTC shortcuts.
- Keep locale preferences and timezone IDs explicit at the API boundary.
- Let MF2 catalogs own wording, plural forms, and word order.

## Features

- `std`, `jiff`, `mf2`, and `icu` are enabled by default.
- `serde` enables serialization support.
- `uniffi` enables UniFFI bindings.

## Contributing

See `CONTRIBUTING.md`.

## License

MIT OR Apache-2.0. See `LICENSE-MIT` and `LICENSE-APACHE`.
