use super::Directive;

impl Directive {
    /// Add the next line to the argument
    ///
    /// # Arguments
    /// `line`: The line to add to the argument.
    /// The line should not have new line characters (`\r\n` or `\n`) at the end.
    ///
    /// # Returns
    /// Returns [`Ok`] if the the format of the line is correct as a continuation of the directive,
    /// and the directive supports multi-line arguments. Otherwise, returns [`Err`].
    pub fn add_line(&mut self, line: &str) -> Result<(), ()> {
        if !self.directive_type.supports_multi_line() {
            return Err(());
        }
        if line.starts_with(&self.whitespaces) {
            let line = &line[self.whitespaces.len()..];
            if line == self.prefix.trim_end_matches(char::is_whitespace) {
                self.args.push("".to_string());
                return Ok(());
            }
            if line.starts_with(&self.prefix) || line.starts_with(&" ".repeat(self.prefix.len())) {
                self.args.push(
                    line[self.prefix.len()..]
                        .trim_end_matches(char::is_whitespace)
                        .to_string(),
                );
                return Ok(());
            }
        }
        Err(())
    }
}

#[cfg(test)]
mod ut {
    use super::super::DirectiveType;
    use super::Directive;

    #[test]
    fn test_addln_empty() {
        let expected = Directive::new(
            "    ",
            "",
            DirectiveType::Empty,
            vec!["ababa".to_string(), "hellow".to_string()],
        );
        let mut directive = Directive::detect_from("    TXTPP# ababa").unwrap();
        assert!(directive.add_line("    hellow").is_ok());
        assert_eq!(expected, directive);
    }

    #[test]
    fn test_addln_empty_add_empty() {
        let expected = Directive::new(
            "    ",
            "",
            DirectiveType::Empty,
            vec!["ababa".to_string(), "".to_string()],
        );
        let mut directive = Directive::detect_from("    TXTPP# ababa").unwrap();
        assert!(directive.add_line("    ").is_ok());
        assert_eq!(expected, directive);
    }

    #[test]
    fn test_addln_empty_format_prefix_ok() {
        let expected = Directive::new(
            " \t \t ",
            "prefix",
            DirectiveType::Empty,
            vec!["ababa\\".to_string(), " hellow".to_string()],
        );
        let mut directive = Directive::detect_from(" \t \t prefixTXTPP# ababa\\").unwrap();
        assert!(directive.add_line(" \t \t prefix hellow").is_ok());
        assert_eq!(expected, directive);
    }

    #[test]
    fn test_addln_empty_format_prefix_notok() {
        let expected = Directive::new(
            " \t \t ",
            "prefix",
            DirectiveType::Empty,
            vec!["ababa\\".to_string()],
        );
        let mut directive = Directive::detect_from(" \t \t prefixTXTPP# ababa\\").unwrap();
        assert!(directive.add_line(" \t \t prefi hellow").is_err());
        assert_eq!(expected, directive);
    }

    #[test]
    fn test_addln_empty_format_spaces_ok() {
        let expected = Directive::new(
            " \t \t ",
            "prefix",
            DirectiveType::Empty,
            vec!["ababa\\".to_string(), "hellow".to_string()],
        );
        let mut directive = Directive::detect_from(" \t \t prefixTXTPP# ababa\\").unwrap();
        assert!(directive.add_line(" \t \t       hellow").is_ok());
        assert_eq!(expected, directive);
    }

    #[test]
    fn test_addln_empty_format_spaces_notok() {
        let expected = Directive::new(
            " \t \t ",
            "prefix",
            DirectiveType::Empty,
            vec!["ababa\\".to_string()],
        );
        let mut directive = Directive::detect_from(" \t \t prefixTXTPP# ababa\\").unwrap();
        assert!(directive.add_line(" \t        hellow").is_err());
        assert_eq!(expected, directive);
    }

    #[test]
    fn test_addln_run() {
        let expected = Directive::new(
            "    ",
            "",
            DirectiveType::Run,
            vec!["ababa\\".to_string(), "hellow".to_string()],
        );
        let mut directive = Directive::detect_from("    TXTPP#run ababa\\").unwrap();
        assert!(directive.add_line("    hellow").is_ok());
        assert_eq!(expected, directive);
    }

    #[test]
    fn test_addln_run_trailing_whitespaces() {
        let expected = Directive::new(
            "    ",
            "",
            DirectiveType::Run,
            vec!["ababa\\".to_string(), "hellowa".to_string()],
        );
        let mut directive = Directive::detect_from("    TXTPP#run ababa\\").unwrap();
        assert!(directive.add_line("    hellowa  \t \t \t").is_ok());
        assert_eq!(expected, directive);
    }

    #[test]
    fn test_addln_run_prefix_trailing_whitespace_notok() {
        let expected = Directive::new(
            "    ",
            "// ",
            DirectiveType::Run,
            vec!["ababa\\".to_string()],
        );
        let mut directive = Directive::detect_from("    // TXTPP#run ababa\\").unwrap();
        assert!(directive.add_line("    //hellowa  \t \t \t").is_err());
        assert_eq!(expected, directive);
    }

    #[test]
    fn test_addln_run_prefix_trailing_whitespace_ok() {
        let expected = Directive::new(
            "    ",
            "// ",
            DirectiveType::Run,
            vec!["ababa\\".to_string(), "".to_string()],
        );
        let mut directive = Directive::detect_from("    // TXTPP#run ababa\\").unwrap();
        assert!(directive.add_line("    //").is_ok());
        assert_eq!(expected, directive);
    }

    #[test]
    fn test_addln_include_single_line_only() {
        let expected = Directive::new(
            "    ",
            "",
            DirectiveType::Include,
            vec!["ababa\\".to_string()],
        );
        let mut directive = Directive::detect_from("    TXTPP#include ababa\\").unwrap();
        assert!(directive.add_line("    hellow").is_err());
        assert_eq!(expected, directive);
    }

    #[test]
    fn test_addln_tag_single_line_only() {
        let expected = Directive::new("    ", "", DirectiveType::Tag, vec!["ababa\\".to_string()]);
        let mut directive = Directive::detect_from("    TXTPP#tag ababa\\").unwrap();
        assert!(directive.add_line("    hellow").is_err());
        assert_eq!(expected, directive);
    }
}
