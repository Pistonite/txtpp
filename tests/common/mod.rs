//! Common utils for integration tests

use copy_dir::copy_dir;
use error_stack::Result;
use murmur3::murmur3_32;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use txtpp::{error::TxtppError, *};

pub struct ItEnv {
    pub cfg: Config,
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
            std::fs::create_dir_all(root_path).unwrap();
        }
        let path = root_path.join(test_name);
        if path.exists() {
            std::fs::remove_dir_all(&path).unwrap();
        }

        // copy example directory to test directory
        copy_dir(example_dir_name, &path).unwrap();

        let config: Config = txtpp::Config {
            num_threads: 8,
            verbosity: Verbosity::Quiet,
            base_dir: path.clone(),
            ..Default::default()
        };

        Self {
            test_description,
            test_dir: path,
            cfg: config,
        }
    }

    #[inline]
    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        f(self)
    }

    #[inline]
    pub fn cfg_mut(&mut self) -> &mut Config {
        &mut self.cfg
    }

    #[inline]
    pub fn run(&self) -> Result<(), TxtppError> {
        txtpp(self.cfg.clone())
    }

    #[inline]
    pub fn set_file(&self, file_name: &str, contents: &str) {
        let path = self.test_dir.join(file_name);
        std::fs::write(path, contents).unwrap();
    }

    #[inline]
    #[allow(dead_code)]
    pub fn delete_file(&self, file_name: &str) {
        let path = self.test_dir.join(file_name);
        std::fs::remove_file(path).unwrap();
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
        let expected = std::fs::read_to_string(&expected_path).unwrap();
        assert_eq!(
            actual,
            expected,
            "file comparions failed in test `{}` ({}) for `{}`",
            self.test_description,
            self.test_dir.display(),
            expected_path.display()
        );
    }

    #[inline]
    pub fn assert_path_exists(&self, path_name: &str, exists: bool) {
        let p = self.test_dir.join(path_name);
        // The file may not be immediately deleted, so we wait a bit if asserting false
        // (can try turning this on if tests fail)
        // if !exists {
        //     for _ in 0..10 {
        //         if !p.exists() {
        //             break;
        //         }
        //         thread::sleep(Duration::from_millis(500));
        //     }
        // }
        assert_eq!(
            p.exists(),
            exists,
            "file existence test failed in test `{}` ({}) for `{path_name}`",
            self.test_description,
            self.test_dir.display()
        );
    }

    pub fn get_modification_time(&self, path_name: &str) -> std::io::Result<std::time::SystemTime> {
        let p = self.test_dir.join(path_name);
        std::fs::metadata(p)?.modified()
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
