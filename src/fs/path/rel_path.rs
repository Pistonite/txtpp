// use super::Error;
// use std::path::Path;

// /// The path that can represent the path to the current file being processed relative to the working directory
// /// 
// /// The path is always relative
// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub struct RelPath {
//     p: String
// }

// impl RelPath {
//     fn as_path(&self) -> &Path {
//         Path::new(&self.p)
//     }

//     fn as_str(&self) -> &str {
//         &self.p
//     }
// }

// impl TryFrom<&str> for RelPath {
//     type Error = Error;

//     /// Convert from a string. Will error if the path is not relative
//     fn try_from(p: &str) -> Result<Self, Self::Error> {
//         let path = Path::new(if p.is_empty() { "." } else { p });
//         if path.is_absolute() {
//             return Err(Error {
//                 offender: p.to_string(),
//                 reason: "Path is not relative".to_string(),
//             });
//         }
//         Ok(Self { p: path.display().to_string() })
//     }
// }

// impl AsRef<Path> for RelPath {
//     fn as_ref(&self) -> &Path {
//         self.as_path()
//     }
// }

// impl AsRef<str> for RelPath {
//     fn as_ref(&self) -> &str {
//         self.as_str()
//     }
// }


// #[cfg(test)]
// mod ut {
//     use super::*;

//     #[test]
//     fn test_dot_path_empty() {
//         let path = RelPath::try_from("").unwrap();
//         assert_eq!(path.p, ".");
//     }

//     #[test]
//     fn test_dot_path_dot() {
//         let path = RelPath::try_from(".").unwrap();
//         assert_eq!(path.p, ".");
//         let path = RelPath::try_from("./").unwrap();
//         assert_eq!(path.p, "./");
//     }

//     #[test]
//     fn test_dot_path_one_seg() {
//         let path = RelPath::try_from("abc").unwrap();
//         assert_eq!(path.p, "abc");
//     }

//     #[test]
//     fn test_dot_path_many_seg() {
//         let path = RelPath::try_from("a/b/c").unwrap();
//         assert_eq!(path.p, "a/b/c");
//     }
//     #[test]
//     fn test_dot_path_dot_seg() {
//         let path = RelPath::try_from("./abc").unwrap();
//         assert_eq!(path.p, "./abc");
//     }

//     #[test]
//     fn test_dot_path_dot_many_seg() {
//         let path = RelPath::try_from("./b/c").unwrap();
//         assert_eq!(path.p, "./b/c");
//     }

//     #[test]
//     fn test_dot_path_dot_many_seg_dot() {
//         let path = RelPath::try_from(".a/b/c").unwrap();
//         assert_eq!(path.p, ".a/b/c");
//     }

//     #[test]
//     fn test_dot_path_parent() {
//         let path = RelPath::try_from("..").unwrap();
//         assert_eq!(path.p, "..");
//         let path = RelPath::try_from("../").unwrap();
//         assert_eq!(path.p, "../");
//     }
// }