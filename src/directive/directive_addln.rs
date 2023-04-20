use super::*;

impl Directive {
    /// Add the next line to the argument
    ///
    /// The line should not have new line characters (`\r\n` or `\n`) at the end.
    /// 
    /// Returns either `Result::Directive` or `Result::NeedNextLine`
    pub fn addln(mut self, line: &str) -> Result {
        let (need_more_lines, line) = if line.ends_with('\\') {
            (true, &line[..line.len() - 1])
        } else {
            (false, line.trim_end_matches(char::is_whitespace))
        };
        if line.starts_with(&self.whitespaces) {
            let line = &line[self.whitespaces.len()..];
            if line.starts_with(&self.prefix) || line.starts_with(&" ".repeat(self.prefix.len())) {
                self.arg.push_str(&line[self.prefix.len()..]);
                return if need_more_lines {
                    Result::NeedNextLine(self)
                } else {
                    Result::Directive(self)
                };
            }
        }
        self.arg.push_str(line);
        return if need_more_lines {
            Result::NeedNextLine(self)
        } else {
            Result::Directive(self)
        };
    }
}

#[cfg(test)]
mod ut {
    use super::*;

    #[test]
    fn test_addln_empty() {
        let expected = Directive::new("    ", "", DirectiveType::Empty, "hellow");
        let directive = Directive::detect_from("    TXTPP# ababa\\")
            .unwrap()
            .addln("hellow");
        assert_eq!(directive, Result::Directive(expected));
    }

    #[test]
    fn test_addln_empty_add_empty() {
        let expected = Directive::new("    ", "", DirectiveType::Empty, "");
        let directive = Directive::detect_from("    TXTPP# ababa\\").unwrap().addln("");
        assert_eq!(directive, Result::Directive(expected));
    }

    #[test]
    fn test_addln_empty_format_prefix_ok() {
        let expected = Directive::new(" \t \t ", "prefix", DirectiveType::Empty, " hellow");
        let directive = Directive::detect_from(" \t \t prefixTXTPP# ababa\\")
            .unwrap()
            .addln(" \t \t prefix hellow");
        assert_eq!(directive, Result::Directive(expected));
    }

    #[test]
    fn test_addln_empty_format_prefix_notok() {
        let expected = Directive::new(
            " \t \t ",
            "prefix",
            DirectiveType::Empty,
            " \t \t prefi hellow",
        );
        let directive = Directive::detect_from(" \t \t prefixTXTPP# ababa\\")
            .unwrap()
            .addln(" \t \t prefi hellow");
        assert_eq!(directive, Result::Directive(expected));
    }

    #[test]
    fn test_addln_empty_format_spaces_ok() {
        let expected = Directive::new(" \t \t ", "prefix", DirectiveType::Empty, " hellow");
        let directive = Directive::detect_from(" \t \t prefixTXTPP# ababa\\")
            .unwrap()
            .addln(" \t \t        hellow");
        assert_eq!(directive, Result::Directive(expected));
    }

    #[test]
    fn test_addln_empty_format_spaces_notok() {
        let expected = Directive::new(
            " \t \t ",
            "prefix",
            DirectiveType::Empty,
            " \t        hellow",
        );
        let directive = Directive::detect_from(" \t \t prefixTXTPP# ababa\\")
            .unwrap()
            .addln(" \t        hellow");
        assert_eq!(directive, Result::Directive(expected));
    }

    #[test]
    fn test_addln_include() {
        let expected = Directive::new("    ", "", DirectiveType::Include, "ababahellow");
        let directive = Directive::detect_from("    TXTPP#include ababa\\")
            .unwrap()
            .addln("hellow");
        assert_eq!(directive, Result::Directive(expected));
    }

    #[test]
    fn test_addln_include_add_empty() {
        let expected = Directive::new("    ", "", DirectiveType::Include, "ababa");
        let directive = Directive::detect_from("    TXTPP#include ababa\\")
            .unwrap()
            .addln("");
        assert_eq!(directive, Result::Directive(expected));
    }

    #[test]
    fn test_addln_include_format_prefix_ok() {
        let expected = Directive::new(" \t \t ", "prefix", DirectiveType::Include, "ababa hellow");
        let directive = Directive::detect_from(" \t \t prefixTXTPP#include ababa\\")
            .unwrap()
            .addln(" \t \t prefix hellow");
        assert_eq!(directive, Result::Directive(expected));
    }

    #[test]
    fn test_addln_include_format_prefix_notok() {
        let expected = Directive::new(
            " \t \t ",
            "prefix",
            DirectiveType::Include,
            "ababa \t \t prefi hellow",
        );
        let directive = Directive::detect_from(" \t \t prefixTXTPP#include ababa\\")
            .unwrap()
            .addln(" \t \t prefi hellow");
        assert_eq!(directive, Result::Directive(expected));
    }

    #[test]
    fn test_addln_include_format_spaces_ok() {
        let expected = Directive::new(" \t \t ", "prefix", DirectiveType::Include, "ababa hellow");
        let directive = Directive::detect_from(" \t \t prefixTXTPP#include ababa\\")
            .unwrap()
            .addln(" \t \t        hellow");
        assert_eq!(directive, Result::Directive(expected));
    }

    #[test]
    fn test_addln_include_format_spaces_notok() {
        let expected = Directive::new(
            " \t \t ",
            "prefix",
            DirectiveType::Include,
            "ababa \t        hellow",
        );
        let directive = Directive::detect_from(" \t \t prefixTXTPP#include ababa\\")
            .unwrap()
            .addln(" \t        hellow  ");
        assert_eq!(directive, Result::Directive(expected));
    }
    #[test]
    fn test_addln_include_multiline() {
        let expected = Directive::new("    ", "", DirectiveType::Include, "ababahellow");
        let directive = Directive::detect_from("    TXTPP#include ababa\\")
            .unwrap()
            .addln("hellow\\");
        assert_eq!(directive, Result::NeedNextLine(expected));
    }

    #[test]
    fn test_addln_include_add_empty_multiline() {
        let expected = Directive::new("    ", "", DirectiveType::Include, "ababa");
        let directive = Directive::detect_from("    TXTPP#include ababa\\")
            .unwrap()
            .addln("\\");
        assert_eq!(directive, Result::NeedNextLine(expected));
    }

    #[test]
    fn test_addln_include_format_prefix_ok_multiline() {
        let expected = Directive::new(" \t \t ", "prefix", DirectiveType::Include, "ababa hellow");
        let directive = Directive::detect_from(" \t \t prefixTXTPP#include ababa\\")
            .unwrap()
            .addln(" \t \t prefix hellow\\");
        assert_eq!(directive, Result::NeedNextLine(expected));
    }

    #[test]
    fn test_addln_include_format_prefix_notok_multiline() {
        let expected = Directive::new(
            " \t \t ",
            "prefix",
            DirectiveType::Include,
            "ababa \t \t prefi hellow",
        );
        let directive = Directive::detect_from(" \t \t prefixTXTPP#include ababa\\")
            .unwrap()
            .addln(" \t \t prefi hellow\\");
        assert_eq!(directive, Result::NeedNextLine(expected));
    }

    #[test]
    fn test_addln_include_format_spaces_ok_multiline() {
        let expected = Directive::new(" \t \t ", "prefix", DirectiveType::Include, "ababa hellow");
        let directive = Directive::detect_from(" \t \t prefixTXTPP#include ababa\\")
            .unwrap()
            .addln(" \t \t        hellow\\");
        assert_eq!(directive, Result::NeedNextLine(expected));
    }

    #[test]
    fn test_addln_include_format_spaces_notok_multiline() {
        let expected = Directive::new(
            " \t \t ",
            "prefix",
            DirectiveType::Include,
            "ababa \t        hellow",
        );
        let directive = Directive::detect_from(" \t \t prefixTXTPP#include ababa\\")
            .unwrap()
            .addln(" \t        hellow\\");
        assert_eq!(directive, Result::NeedNextLine(expected));
    }

    #[test]
    fn test_addln_include_format_spaces_ok_multiline_trailing_whitespaces() {
        let expected = Directive::new(
            " \t \t ",
            "prefix",
            DirectiveType::Include,
            "ababa hellow\\",
        );
        let directive = Directive::detect_from(" \t \t prefixTXTPP#include ababa\\")
            .unwrap()
            .addln(" \t \t        hellow\\  ");
        assert_eq!(directive, Result::Directive(expected));
    }

    #[test]
    fn test_addln_include_format_spaces_ok_multiline_trailing_whitespaces_before_backslash() {
        let expected = Directive::new(
            " \t \t ",
            "prefix",
            DirectiveType::Include,
            "ababa hellow  ",
        );
        let directive = Directive::detect_from(" \t \t prefixTXTPP#include ababa\\")
            .unwrap()
            .addln(" \t \t        hellow  \\");
        assert_eq!(directive, Result::NeedNextLine(expected));
    }
}
