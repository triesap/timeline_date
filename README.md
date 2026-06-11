# timeline_date

Localised timeline date labels for Rust applications.

## Goals

- Format feed, detail, and audit labels from explicit event and current timestamps.
- Classify dates with user-timezone civil-day semantics, not UTC shortcuts.
- Keep locale preferences and timezone IDs explicit at the API boundary.
- Let MF2 catalogs own wording, plural forms, and word order.

## Localisation model

`timeline_date` is designed around MF2 message catalogs, Jiff timezone math,
and ICU4X date/time formatting.

## Contributing

See `CONTRIBUTING.md`.

## License

MIT OR Apache-2.0. See `LICENSE-MIT` and `LICENSE-APACHE`.
