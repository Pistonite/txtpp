//! Common utils for integration tests
//!
//!

use copy_dir::copy_dir;
use error_stack::Result;
use murmur3::murmur3_32;
use std::{
    io::BufReader,
    path::{Path, PathBuf},
};
use txtpp::{error::TxtppError, *};

pub struct ItEnv {
    config: Config,
    test_description: String,
    test_dir: PathBuf,
}

impl ItEnv {
    pub fn new(example_dir_name: &str) -> Self {
        let test_description = example_dir_name.to_string();
        let mut read = BufReader::new(test_description.as_bytes());
        let test_name = format!(
            "test-{}",
            murmur3_32(&mut read, test_description.len().try_into().unwrap()).unwrap()
        );
        // create test directory
        let root_path = Path::new("target/test_out");
        if !root_path.exists() {
            std::fs::create_dir_all(&root_path).unwrap();
        }
        let path = root_path.join(test_name);
        if path.exists() {
            std::fs::remove_dir_all(&path).unwrap();
        }

        // copy example directory to test directory
        copy_dir(example_dir_name, &path).unwrap();

        let mut config = Config::default();
        config.num_threads = 1;
        config.verbosity = Verbosity::Quiet;
        config.base_dir = path.clone();

        Self {
            test_description,
            test_dir: path,
            config,
        }
    }

    #[inline]
    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self) -> (),
    {
        f(self)
    }

    #[inline]
    pub fn cfg(&mut self) -> &mut Config {
        &mut self.config
    }

    #[inline]
    pub fn run(&self) -> Result<(), TxtppError> {
        txtpp(self.config.clone())
    }

    #[inline]
    pub fn set_file(&self, file_name: &str, contents: &str) {
        let path = self.test_dir.join(file_name);
        std::fs::write(&path, contents).unwrap();
    }

    #[inline]
    #[allow(dead_code)]
    pub fn delete_file(&self, file_name: &str) {
        let path = self.test_dir.join(file_name);
        std::fs::remove_file(&path).unwrap();
    }

    pub fn assert_file_eq(&self, file_name: &str, expected_file_name: &str) {
        let expected_path = self.test_dir.join(expected_file_name);
        assert!(
            expected_path.exists(),
            "expected file `{}` does not exist in test `{}` ({})",
            expected_file_name,
            self.test_description,
            self.test_dir.display()
        );
        let actual_path = self.test_dir.join(file_name);
        assert!(
            actual_path.exists(),
            "expected output file `{}` does not exist in test `{}` ({})",
            file_name,
            self.test_description,
            self.test_dir.display()
        );
        let actual = std::fs::read_to_string(actual_path).unwrap();
        let expected = std::fs::read_to_string(expected_path).unwrap();
        assert_eq!(
            actual,
            expected,
            "file comparions failed in test `{}` ({})",
            self.test_description,
            self.test_dir.display()
        );
    }

    #[inline]
    pub fn assert_path_exists(&self, path_name: &str, exists: bool) {
        assert_eq!(
            exists,
            self.test_dir.join(path_name).exists(),
            "file existence test failed in test `{}` ({})",
            self.test_description,
            self.test_dir.display()
        );
    }
}

macro_rules! testit {
    ($test_name:ident, $fnonce:expr) => {
        #[test]
        #[allow(non_snake_case)]
        fn $test_name() {
            let mut env = ItEnv::new(&stringify!($test_name).replace("__", "/"));
            env.execute($fnonce);
        }
    };
}

pub(crate) use testit;
