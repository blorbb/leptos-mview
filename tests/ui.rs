#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/errors/*.rs");
    t.pass("tests/ui/pass/*.rs");
}
