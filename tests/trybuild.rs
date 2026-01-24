#[test]
#[cfg(not(miri))]
fn trybuild_test() {
    let tc = trybuild::TestCases::new();
    tc.compile_fail("tests/ui/**/*.rs");
}
