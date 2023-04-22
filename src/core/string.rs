pub trait ReplaceLineEnding {
    fn replace_line_ending(&self, line_ending: &str, force_trailing_newline: bool) -> String;
}

impl ReplaceLineEnding for str {
    fn replace_line_ending(&self, line_ending: &str, force_trailing_newline: bool) -> String {
        let has_trailing_newline = force_trailing_newline || self.ends_with('\n');
        let mut result = String::new();
        let lines = self.lines().into_iter().collect::<Vec<_>>();
        if let Some(last) = lines.last() {
            for line in &lines[..lines.len() - 1] {
                result.push_str(line);
                result.push_str(line_ending);
            }
            result.push_str(last);
        }
                
        if has_trailing_newline {
            result.push_str(line_ending);
        }
        result
    }
}

#[cfg(test)]
mod ut {
    use super::*;

    #[test]
    fn test_empty() {
        assert_eq!("", "".replace_line_ending("\n", false));
        assert_eq!("", "\r".replace_line_ending("\n", false));
        assert_eq!("\n", "\r\n".replace_line_ending("\n", false));
    }

    #[test]
    fn test_empty_force_lf() {
        assert_eq!("\n", "".replace_line_ending("\n", true));
    }

    #[test]
    fn test_empty_force_crlf() {
        assert_eq!("\r\n", "".replace_line_ending("\r\n", true));
    }

    #[test]
    fn test_one() {
        assert_eq!("a", "a".replace_line_ending("\n", false));
    }

    #[test]
    fn test_one_force_lf() {
        assert_eq!("a\n", "a".replace_line_ending("\n", true));
        assert_eq!("a\n", "a\n".replace_line_ending("\n", true));
        assert_eq!("a\n", "a\r\n".replace_line_ending("\n", true));
    }
    #[test]
    fn test_multiple_empty_lines_crlf() {
        assert_eq!("a\r\n\r\n", "a\n\n".replace_line_ending("\r\n", false));
        assert_eq!("a\r\n\r\na\r\n", "a\n\na".replace_line_ending("\r\n", true));
        assert_eq!("a\r\n\r\r\n", "a\r\n\r\r\n".replace_line_ending("\r\n", false));
    }
}