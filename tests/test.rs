mod common;
use common::*;

use txtpp::*;

testit!(examples__include, |env| {
    env.cfg().mode = Mode::Verify;
    // before output
    assert!(env.run().is_err());
    env.cfg().mode = Mode::Build;
    assert!(env.run().is_ok());
    env.assert_file_eq("foo.txt", "foo.txt.expected");
    env.assert_file_eq("biz.txt", "bar.txt");
    env.cfg().mode = Mode::Verify;
    assert!(env.run().is_ok());
    env.set_file("foo.txt", "not the same");
    assert!(env.run().is_err());
    env.cfg().mode = Mode::Clean;
    assert!(env.run().is_ok());
    env.assert_path_exists("foo.txt", false);
});

#[cfg(not(windows))]
testit!(examples__run, |env| {
    env.cfg().mode = Mode::Build;
    env.cfg().inputs = vec!["foo.txt.txtpp".to_string()];
    assert!(env.run().is_ok());
    env.assert_file_eq("foo.txt", "foo.txt.expected");
    env.cfg().inputs = vec!["invalid.txtpp".to_string()];
    assert!(env.run().is_err());
});

testit!(examples__circular_dep, |env| {
    env.cfg().mode = Mode::Build;
    env.cfg().inputs = vec!["a.txt".to_string()];
    assert!(env.run().is_err());
    env.assert_file_eq("c.txt", "c.txt.expected");
});

testit!(examples__temp__write_clean, |env| {
    assert!(env.run().is_ok());
    // content should match
    env.assert_file_eq("exp", "exp.expected");
    env.assert_file_eq("temp.out", "temp.out.expected");
    // but should when running clean
    env.cfg().mode = Mode::Clean;
    assert!(env.run().is_ok());
    env.assert_path_exists("temp.out", false);
    env.assert_path_exists("exp.txt", false);
});

// #[test]
// fn test_include_simple_verify_fail() {

// }
