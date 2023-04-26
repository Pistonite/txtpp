mod common;
use common::ItEnv;

use txtpp::*;
#[test]
fn test_include_simple_build_verify_clean() {
    let mut env = ItEnv::new("test_include_simple_build_verify_clean", "examples/include");
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
}

#[cfg(not(windows))]
#[test]
fn test_run_simple_build() {
    let mut env = ItEnv::new("test_run_simple_build", "examples/run");
    env.cfg().mode = Mode::Build;
    env.cfg().inputs = vec!["foo.txt.txtpp".to_string()];
    assert!(env.run().is_ok());
    env.assert_file_eq("foo.txt", "foo.txt.expected");
    env.cfg().inputs = vec!["invalid.txtpp".to_string()];
    assert!(env.run().is_err());
}

#[test]
fn test_circular_dep() {
    let mut env = ItEnv::new("test_circular_dep", "examples/circular_dep");
    env.cfg().mode = Mode::Build;
    env.cfg().inputs = vec!["a.txt".to_string()];
    assert!(env.run().is_err());
    env.assert_file_eq("c.txt", "c.txt.expected");
}



// #[test]
// fn test_include_simple_verify_fail() {

// }