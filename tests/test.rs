mod common;
use common::*;

use txtpp::*;

testit!(examples__empty_test, |env| {
    assert!(env.run().is_ok());
    env.assert_file_eq("empty1", "empty1.txtpp");
    env.assert_file_eq("emptyempty", "emptyempty.txtpp");
});

testit!(examples__include, |env| {
    env.cfg.mode = Mode::Verify;
    // before output
    assert!(env.run().is_err());
    env.cfg.mode = Mode::Build;
    assert!(env.run().is_ok());
    env.assert_file_eq("foo.txt", "foo.txt.expected");
    env.assert_file_eq("biz.txt", "biz.txt.expected");
    env.cfg.mode = Mode::Verify;
    assert!(env.run().is_ok());
    env.set_file("foo.txt", "not the same");
    assert!(env.run().is_err());
    env.cfg.mode = Mode::Clean;
    assert!(env.run().is_ok());
    env.assert_path_exists("foo.txt", false);
    // non-existent target should error
    env.cfg.mode = Mode::Build;
    env.cfg.inputs = vec!["a.txt".to_string()];
    assert!(env.run().is_err());
    env.cfg.inputs = vec!["a.txtpp".to_string()];
    assert!(env.run().is_err());
});

#[cfg(not(windows))]
testit!(examples__multiple_include, |env| {
    assert!(env.run().is_ok());
    env.assert_file_eq("a", "a.expected");
    env.assert_file_eq("b", "b.expected");
    env.assert_file_eq("c", "c.expected");
});

#[cfg(windows)]
testit!(examples__multiple_include_windows, |env| {
    assert!(env.run().is_ok());
    env.assert_file_eq("a", "a.expected");
    env.assert_file_eq("b", "b.expected");
    env.assert_file_eq("c", "c.expected");
});

testit!(examples__run, |env| {
    env.cfg.mode = Mode::Build;
    env.cfg.inputs = vec!["foo.txt.txtpp".to_string()];
    assert!(env.run().is_ok());
    env.assert_file_eq("foo.txt", "foo.txt.expected");
    env.cfg.inputs = vec!["invalid.txtpp".to_string()];
    assert!(env.run().is_err());
});

testit!(examples__circular_dep, |env| {
    env.cfg.mode = Mode::Build;
    env.cfg.inputs = vec!["a.txt".to_string()];
    assert!(env.run().is_err());
    env.assert_file_eq("c.txt", "c.txt.expected");
});

testit!(examples__circular_dep_self, |env| {
    assert!(env.run().is_err());
    env.assert_file_eq("invalid", "invalid.expected");
});

testit!(examples__temp__write_clean, |env| {
    assert!(env.run().is_ok());
    // content should match
    env.assert_file_eq("exp", "exp.expected");
    env.assert_file_eq("temp.out", "temp.out.expected");
    env.cfg.mode = Mode::Clean;
    assert!(env.run().is_ok());
    env.assert_path_exists("temp.out", false);
    env.assert_path_exists("exp", false);
    // verify should fail fast, so temp.out is not created
    env.cfg.mode = Mode::Verify;
    assert!(env.run().is_err());
    env.set_file("exp", "not the same");
    assert!(env.run().is_err());
    env.assert_path_exists("temp.out", false);
    env.set_file(
        "exp",
        "This example writes to a temp file, then include it\nThis line should be in the output",
    );
    // since the first part is the same, it will create the temp file
    assert!(env.run().is_err());
    env.assert_path_exists("temp.out", true);
});

testit!(examples__temp__python_script, |env| {
    assert!(env.run().is_ok());
    env.assert_file_eq("city.js", "city.js.expected");
    env.assert_file_eq("gen_cities.g.py", "gen_cities.g.py.expected");
    env.cfg.mode = Mode::Clean;
    assert!(env.run().is_ok());
    env.assert_path_exists("city.js", false);
    env.assert_path_exists("gen_cities.g.py", false);
});

testit!(examples__trailing_newline, |env| {
    assert!(env.run().is_ok());
    env.assert_file_eq("source", "source.expected.trailing");
    env.cfg.trailing_newline = false;
    assert!(env.run().is_ok());
    env.assert_file_eq("source", "source.expected");
});

testit!(examples__tag__inject_one, |env| {
    assert!(env.run().is_ok());
    env.assert_file_eq("a", "a.expected");
});

#[cfg(linux)]
testit!(examples__tag__inject_serial, |env| {
    assert!(env.run().is_ok());
    env.assert_file_eq("b.html", "b.html.expected");
});

#[cfg(windows)]
testit!(examples__tag__inject_serial_windows, |env| {
    assert!(env.run().is_ok());
    env.assert_file_eq("b.html", "b.html.expected");
});

#[cfg(linux)]
testit!(examples__tag__inject_multiple, |env| {
    assert!(env.run().is_ok());
    env.assert_file_eq("test.txt", "test.txt.expected");
});

#[cfg(windows)]
testit!(examples__tag__inject_multiple_windows, |env| {
    assert!(env.run().is_ok());
    env.assert_file_eq("test.txt", "test.txt.expected");
});

testit!(examples__tag__no_output_skip, |env| {
    assert!(env.run().is_ok());
    env.assert_file_eq("test", "test.expected");
    env.assert_file_eq("temp", "temp.expected");
});

testit!(examples__tag__error, |env| {
    env.cfg.inputs = vec!["tag_multiple_same".to_string()];
    assert!(env.run().is_err());
    env.cfg.inputs = vec!["tag_multiple_prefix".to_string()];
    assert!(env.run().is_err());
    env.cfg.inputs = vec!["tag_multiple_prefix2".to_string()];
    assert!(env.run().is_err());
});

testit!(examples__tag__inject_once, |env| {
    assert!(env.run().is_ok());
    env.assert_file_eq("test", "test.expected");
});

testit!(examples__tag__unused_at_end, |env| {
    assert!(env.run().is_err());
});

testit!(examples__tag__complex, |env| {
    assert!(env.run().is_ok());
    env.assert_file_eq("test", "test.expected");
});
