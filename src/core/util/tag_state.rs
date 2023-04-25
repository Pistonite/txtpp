use crate::core::ReplaceLineEnding;
use error_stack::{Report, Result};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub struct TagState {
    listening: Option<String>,
    stored: HashMap<String, String>,
}

#[derive(Debug)]
pub struct TagStateError;

impl Display for TagStateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tag state is invalid")
    }
}

impl Error for TagStateError {}

impl TagState {
    pub fn new() -> Self {
        Self {
            listening: None,
            stored: HashMap::new(),
        }
    }

    pub fn create(&mut self, tag: &str) -> Result<(), TagStateError> {
        if let Some(old_tag) = &self.listening {
            return Err(Report::new(TagStateError).attach_printable(format!(
                "Cannot create new tag {tag} when old tag {old_tag} is still listening"
            )));
        }
        for k in self.stored.keys() {
            if k.starts_with(tag) || tag.starts_with(k) {
                if k == &tag {
                    return Err(Report::new(TagStateError).attach_printable(format!(
                        "Cannot create new tag {tag} when a tag with the same name is still listening"
                    )));
                }
                return Err(Report::new(TagStateError).attach_printable(format!(
                    "Cannot create new tag {tag} because it is ambiguous with existing tag {k}"
                )));
            }
        }

        self.listening = Some(tag.to_string());
        Ok(())
    }

    pub fn try_store(&mut self, content: &str) -> std::result::Result<(), ()> {
        match &self.listening {
            Some(tag) => {
                self.stored.insert(tag.clone(), content.to_string());
                self.listening = None;
                Ok(())
            }
            None => Err(()),
        }
    }

    pub fn inject_tags(&mut self, output: &str, line_ending: &str) -> String {
        let mut to_inject = self
            .stored
            .iter()
            .map(|(k, v)| output.find(k).map(|i| (i, k, v)))
            .flatten()
            .collect::<Vec<_>>();
        // sort by index
        to_inject.sort_by(|a, b| a.0.cmp(&b.0));
        let mut injected_output = String::new();
        let mut last_end = 0;
        let mut to_remove = vec![];
        for (i, key, value) in &to_inject {
            if *i < last_end {
                continue;
            }
            injected_output.push_str(&output[last_end..*i]);
            injected_output.push_str(&value.replace_line_ending(line_ending, false));
            last_end = i + key.len();
            to_remove.push(key.to_string());
        }
        for key in to_remove {
            self.stored.remove(&key);
        }
        injected_output.push_str(&output[last_end..]);
        injected_output.push_str(line_ending);
        injected_output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create_ok() {
        let mut tag_state = TagState::new();
        tag_state.create("tag1").unwrap();
        assert_eq!(Some("tag1".to_string()), tag_state.listening);
    }

    #[test]
    fn test_create_already_listening() {
        let mut tag_state = TagState::new();
        tag_state.create("tag1").unwrap();
        assert!(tag_state.create("tag2").is_err());
    }

    #[test]
    fn test_create_has_same_listening() {
        let mut tag_state = TagState::new();
        tag_state.create("tag1").unwrap();
        assert!(tag_state.create("tag1").is_err());
    }

    #[test]
    fn test_create_has_prefix_listening() {
        let mut tag_state = TagState::new();
        tag_state.create("tag1").unwrap();
        assert!(tag_state.create("tag").is_err());
    }

    #[test]
    fn test_create_has_same_stored() {
        let mut tag_state = TagState::new();
        tag_state.create("tag1").unwrap();
        tag_state.try_store("content1").unwrap();
        assert!(tag_state.create("tag1").is_err());
    }

    #[test]
    fn test_create_has_prefix_stored() {
        let mut tag_state = TagState::new();
        tag_state.create("tag1").unwrap();
        tag_state.try_store("content1").unwrap();
        assert!(tag_state.create("tag").is_err());
    }

    #[test]
    fn test_create_prefix_stored() {
        let mut tag_state = TagState::new();
        tag_state.create("tag").unwrap();
        tag_state.try_store("content1").unwrap();
        assert!(tag_state.create("tag1").is_err());
    }

    #[test]
    fn test_try_store_ok() {
        let mut tag_state = TagState::new();
        tag_state.create("tag1").unwrap();
        assert!(tag_state.try_store("content1").is_ok());
        assert_eq!(None, tag_state.listening);
        assert_eq!(
            Some("content1".to_string()),
            tag_state.stored.get("tag1").cloned()
        );
    }

    #[test]
    fn test_try_store_no_listening() {
        let mut tag_state = TagState::new();
        assert!(tag_state.try_store("content1").is_err());
    }

    fn create_test_state() -> TagState {
        let mut tag_state = TagState::new();
        tag_state.create("tag1").unwrap();
        tag_state.try_store("content1").unwrap();
        tag_state.create("tag2").unwrap();
        tag_state.try_store("content2").unwrap();
        tag_state.create("tag3").unwrap();
        tag_state.try_store("content3").unwrap();
        tag_state
    }

    #[test]
    fn test_tag_inject_one_all() {
        let mut tag_state = create_test_state();
        let output = tag_state.inject_tags("tag1", "\n");
        assert_eq!("content1\n", &output);
        assert_eq!(2, tag_state.stored.len());
    }

    #[test]
    fn test_tag_inject_one_prefixed() {
        let mut tag_state = create_test_state();
        let output = tag_state.inject_tags("hellotag1", "\n");
        assert_eq!("hellocontent1\n", &output);
        assert_eq!(2, tag_state.stored.len());
    }

    #[test]
    fn test_tag_inject_one_postfixed() {
        let mut tag_state = create_test_state();
        let output = tag_state.inject_tags("tag1hello", "\n");
        assert_eq!("content1hello\n", &output);
        assert_eq!(2, tag_state.stored.len());
    }

    #[test]
    fn test_tag_inject_one_sandwiched() {
        let mut tag_state = create_test_state();
        let output = tag_state.inject_tags("worldtag1hello", "\n");
        assert_eq!("worldcontent1hello\n", &output);
        assert_eq!(2, tag_state.stored.len());
    }

    #[test]
    fn test_tag_inject_one_sandwiched_newlines() {
        let mut tag_state = create_test_state();
        let output = tag_state.inject_tags("worldtag1hello", "\n");
        assert_eq!("worldcontent1hello\n", &output);
        assert_eq!(2, tag_state.stored.len());
    }

    #[test]
    fn test_tag_inject_many() {
        let mut tag_state = create_test_state();
        let output = tag_state.inject_tags("tag1tag2tag3", "\n");
        assert_eq!("content1content2content3\n", &output);
        assert_eq!(0, tag_state.stored.len());
    }

    #[test]
    fn test_tag_inject_none() {
        let mut tag_state = create_test_state();
        let output = tag_state.inject_tags("asdftag 1fg   tag 2tag 3asdf", "\n");
        assert_eq!("asdftag 1fg   tag 2tag 3asdf\n", &output);
        assert_eq!(3, tag_state.stored.len());
    }

    #[test]
    fn test_tag_inject_many_complex() {
        let mut tag_state = create_test_state();
        let output = tag_state.inject_tags("asdftag 1fg   tag2tag3asdf", "\n");
        assert_eq!("asdftag 1fg   content2content3asdf\n", &output);
        assert_eq!(1, tag_state.stored.len());
    }

    #[test]
    fn test_tag_inject_many_complex2() {
        let mut tag_state = TagState::new();
        tag_state.create("tag1").unwrap();
        tag_state.try_store("cont\r\nent1").unwrap();
        tag_state.create("tag2").unwrap();
        tag_state.try_store("\ncontent2\n").unwrap();
        tag_state.create("tag3").unwrap();
        tag_state.try_store("content3\n\n").unwrap();
        let output = tag_state.inject_tags("asdftag1fg   tag2tag3asdf", "\r\n");
        assert_eq!(
            "asdfcont\r\nent1fg   \r\ncontent2\r\ncontent3\r\n\r\nasdf\r\n",
            &output
        );
        assert_eq!(0, tag_state.stored.len());
    }
}
