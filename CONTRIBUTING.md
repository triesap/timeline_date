# Contributing

Thanks for your interest in contributing to timeline_date.

## Ways to help

- Report bugs and regressions
- Improve documentation and examples

## Development setup

This repository is a Rust workspace. Run the full local lane before opening a
pull request:

```sh
cargo fmt --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo test --no-default-features --features std,jiff,mf2
cargo doc --all-features --no-deps
```

## Pull request checklist

- Keep changes focused and well-scoped
- Add or update unit tests when behavior changes
- Keep public APIs documented
- Avoid introducing new unsafe code

## Code style

- Use idiomatic Rust
- Prefer small, composable helpers
- Favor clear, explicit APIs over cleverness

## License

By contributing, you agree that your contributions are released under the
project license (MIT OR Apache-2.0).
