# data-classes

`data-classes` provides a single, ergonomic attribute macro `#[data]` that simplifies common boilerplate for plain data structs and enums.

The `#[data]` macro is the primary feature of this crate. It acts as a concise, configurable shortcut for deriving common traits, controlling ABI/layout, and generating small helper methods (like `new` and `Default`). It can also opt in to integrations such as `serde`, `rkyv`, and `bytemuck` behind feature flags.

## Overview

- Purpose: Reduce repetitive `derive` and small impl boilerplate for simple data types.
- Primary interface: the attribute macro `#[data(...)]` placed on structs or enums.
- Behavior: depending on attributes and crate feature flags, `#[data]` will add `#[repr(...)]`, `#[derive(...)]`, and optionally generate `impl` blocks (e.g. `Default`, `new`, `Display`).

## Key features

- Automatic derives: `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash` are derived by default.
- Layout control: add `repr` options such as `raw` (C), `packed`, `transparent`, or integer-backed `u8/u16/...` via the macro.
- `Default` handling: `#[data(default)]` either derives `Default` or generates a custom `Default` impl when field-level defaults are present.
- Constructor generation: `#[data(new)]` creates a `pub fn new(...) -> Self` for structs with named fields; `#[data(new(default))]` generates `new()` delegating to `Default`.
- Display helpers: `#[data(display(...))]` supports `debug`, `comma`, `semicolon`, and `space` variants to quickly implement `Display`.
- Optional integrations (behind features): `serde` support, `rkyv` support (with additional per-attribute options), and `bytemuck` support (`pod`, `zeroable`).

## Supported attributes

You can combine multiple options inside the `#[data(...)]` attribute. Examples of supported options include:

- `raw` — apply `#[repr(C)]`.
- `packed` — apply `#[repr(packed)]`.
- `transparent` — apply `#[repr(transparent)]`.
- `u8`, `u16`, `u32`, `u64`, `usize`, `i8`, `i16`, `i32`, `i64`, `isize` — use a specific integer `repr`.
- `default` — derive or implement `Default` (field-level defaults may produce a custom `Default`).
- `copy` — derive `Copy`.
- `serde` — (requires `serde` feature) derive `Serialize` and `Deserialize`.
- `rkyv(...)` — (requires `rkyv` feature) enables `rkyv` derives and accepts sub-options like `cmp` and `omit-bounds`.
- `pod`, `zeroable` — (require `bytemuck` feature) derive `bytemuck::Pod` / `bytemuck::Zeroable` and may adjust `repr` as needed.
- `display(debug)` — implement `Display` by formatting with `{:?}`.
- `display(comma|semicolon|space)` — implement `Display` by joining struct fields with `,`, `;`, or space respectively.
- `new` and `new(default)` — generate `new` constructors. Field-level `new_value` and defaults are supported for more fine-grained constructor generation.

## Examples

Use only the default derives.

For example:

```rust
#[data]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}
```

Will expand to:

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}
```

---

Add more options to generate additional code.

For example:

```rust
#[data(new, default, serde, rkyv(cmp), pod)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}
```

Will expand to:

```rust
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize, bytemuck::Pod, bytemuck::Zeroable, Copy)]
#[rkyv(derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash), compare(PartialEq, PartialOrd))]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}
impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
```
