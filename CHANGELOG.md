# Changelog

## v0.2.0 (25/10/2023)

**Breaking Changes:**
- [#4](https://github.com/blorbb/leptos-mview/pull/4): Rename macro from `view!` to `mview!`.

    A full-word search-and-replace for `view!` -> `mview!` and `leptos_mview::view` -> `leptos_mview::mview` should fix everything.

- [#3](https://github.com/blorbb/leptos-mview/pull/3): `clone:` directive now takes a single identifier, `clone:thing` instead of `clone:{thing}`.

**Enhancements:**

- [#3](https://github.com/blorbb/leptos-mview/pull/3): added `use:` directive to elements. Syntax is the same as leptos, `use:dir` or `use:dir={param}`.

## v0.1.0 (30/09/2023)

Initial release 🎉
