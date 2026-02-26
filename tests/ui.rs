#[test]
fn ui() {
    // NOTE:
    // since the view tree is statically typed and uses a bunch of traits, the
    // errors may be very poor. Adding `RUSTFLAGS='--cfg erase_components'` fixes a
    // lot of these as the errors stop at these type erasure boundaries.
    //
    // These should be tested with this cfg flag on. The `.cargo/config.toml`
    // enables this flag. You will need to change the target triple depending on
    // your platform.
    //
    // Files in `ui/*` are not tracked by rust-analyzer.
    // If you want to see what's actually outputted by rust-analyzer, copy it into
    // `tests/tmp.rs` (gitignored).
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/*.rs");
    t.compile_fail("tests/ui/errors/*.rs");
}
