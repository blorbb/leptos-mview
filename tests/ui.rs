#[test]
fn ui() {
    // FIXME: diagnostics are unavoidably bad right now
    // due to trait errors when there is an issue.
    // these trait errors span the entire macro output,
    // so there is no good way to scope errors to a specific span.
    //
    // not running any UI tests for now.

    // let t = trybuild::TestCases::new();
    // t.pass("tests/ui/pass/*.rs");
    // t.compile_fail("tests/ui/errors/*.rs");
}
