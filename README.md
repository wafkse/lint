# Personal Rust lints

A reusable Dylint library for Rust policy lints shared across unrelated projects.

The lint library uses a pinned nightly compiler because Dylint depends on rustc internals. Projects being linted can continue to use stable Rust.

## Lints

`no_numeric_field_access`

Rejects unnamed numeric field access such as `tuple.0` and `wrapper.1`. Named field access and index expressions such as `items[0]` are unaffected.

`no_self_field_access`

Rejects stored field access derived from the method receiver such as `self.rsi`, `(*self).rsi`, and immutable aliases of `self`. Any receiver destructure used to derive stored fields must be the first statement in the method. Method calls on the receiver remain valid.

`self_pattern_rest`

Requires `..` in `let Self(...)` patterns when two or more consecutive plain `_` wildcards occur at the beginning or end. For example, `Self(_, _, value)` becomes `Self(.., value)` and `Self(value, _, _)` becomes `Self(value, ..)`. Explicit reference patterns are included. Single wildcards, interior runs, macro-generated patterns, and patterns already containing `..` are ignored. When both edges qualify, the leading run is reported so only one `..` is introduced.

`private_single_call_fn`

Rejects trivial private free functions and inherent methods that are used only once. Only items with inherited visibility (`fn ...`) are considered; `pub`, `pub(crate)`, `pub(super)`, `pub(self)`, other restricted visibility, and trait methods are ignored. Method-call syntax such as `value.helper()` is counted. A body is considered trivial only when it contains at most six HIR expression nodes and no control flow, closures or async/coroutine constructs, unsafe blocks, inline assembly, yield/await, or early exits.

```rust
fn rsi(&self) -> &Gpr {
    let Self { rsi, .. } = self;

    rsi
}
```

`statement_group_spacing`

Requires a blank physical line between structurally distinct groups inside authored blocks. Consecutive identifier or wildcard `let` bindings may remain together. Structural pattern bindings and `let else` statements stand alone, while expressions and local items form separate groups. A block tail expression participates as an expression group. Comment-only lines do not count as blank separators.

`struct_invariant_note`

Requires every authored struct with at least one non-public field to own exactly one ordinary line comment containing `NOTE(invariant):` immediately above the struct declaration. The marker is rejected on fully public structs and everywhere outside that structure-owned position.

`explicit_restricted_visibility`

Rejects crate-root and parent-only restricted visibility, including `pub(crate)`, `pub(super)`, `pub(in crate)`, and `pub(in super)`. Restricted visibility must name a real ancestor module such as `pub(in crate::subsystem)`. An adjacent ordinary line comment containing `NOTE(rationale):` permits an otherwise prohibited visibility when the broader boundary is intentional.

## Development

Install the Dylint tooling once.

```sh
cargo install cargo-dylint dylint-link
```

The repository pins the compiler required by the lint library in `rust-toolchain.toml`.

```sh
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
```

## Use from another project

For local development, put the library in the consuming workspace `dylint.toml`.

```toml
[workspace.metadata.dylint]
libraries = [
    { path = "<path-to-personal-lints>" },
]
```

For use across machines, replace the path entry with a Git source and pin a tag or revision.

```toml
[workspace.metadata.dylint]
libraries = [
    { git = "<repository-url>", rev = "<commit>" },
]
```

Run every configured lint across the workspace.

```sh
cargo dylint --all -- --workspace --all-targets
```

Use Clippy separately when index expressions such as `items[0]` should also be prohibited.

```sh
cargo clippy --workspace --all-targets -- -D clippy::indexing_slicing
```
