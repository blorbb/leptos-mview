# Changelog

## v0.2.1 (18/11/2023)

Hopefully no breaking changes, just a whole bunch of new DX features!

**Features:**
- Support for slots! Prefix the slot element with `slot:` to use it as a slot. The parameter name in the parent component must be the same name as the slot, in snake_case.
    - See [`Slots`](https://github.com/blorbb/leptos-mview/tree/bc1bd012828e4db6f963ba9296435bdf753ee640#slots) for more details.
- A new value shorthand: `f["{:.3}", some_number()]`: adding the `f` prefix adds `format!` into the closure; equivalent to `[format!(...)]` or `{move || format!(...)}`.
- On components, if you have the parameters `#[prop(into, optional)] class: TextProp` or `#[prop(optional)] id: &'static str`, you can use the selector shorthand `Component.some-class #some-id` to add it to these attributes, and/or the `class:` directive for reactive classes.
    - `impl Default for TextProp` is currently in the Leptos git main / future v0.5.3 release; for 0.5.2 or earlier, you need to use `#[prop(default="".into())]` instead of `optional`.
    - Reactive classes in this way are not very performant as a `String` is constructed every time any of the signals change. If you *only* use the selector shorthand, they will be compiled into one string literal (no performance downsides).
    - This is also supported on slots, using the same fields.
    - See [Special Attributes](https://github.com/blorbb/leptos-mview/tree/bc1bd012828e4db6f963ba9296435bdf753ee640#special-attributes) for more details.

**Enhancements:**
- Fixed up a whole lot of spans to improve a whole bunch of error messages: hopefully, there are no more errors that cause the entire macro to be spanned, errors should always be localized to a relevant spot.
- Better rust-analyzer support in some invalid macro situations.
- Better syntax highlighting for the selector shorthand on HTML elements.

## v0.2.0 (25/10/2023)

**Breaking Changes:**
- [#4](https://github.com/blorbb/leptos-mview/pull/4): Rename macro from `view!` to `mview!`.

    A full-word search-and-replace for `view!` -> `mview!` and `leptos_mview::view` -> `leptos_mview::mview` should fix everything.

- [#3](https://github.com/blorbb/leptos-mview/pull/3): `clone:` directive now takes a single identifier, `clone:thing` instead of `clone:{thing}`.

**Features:**

- [#3](https://github.com/blorbb/leptos-mview/pull/3): added `use:` directive to elements. Syntax is the same as leptos, `use:dir` or `use:dir={param}`.

## v0.1.0 (30/09/2023)

Initial release 🎉
