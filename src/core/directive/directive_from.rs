use super::*;

pub const TXTPP_HASH: &str = "TXTPP#";

impl Directive {
    /// Try detecting a directive from a line
    ///
    /// # Arguments
    /// `line`: The line to detect the directive from.
    /// The line should not have new line characters (`\r\n` or `\n`) at the end.
    ///
    /// # Returns
    /// The detected directive with its first argument from the line,
    /// or [`None`] if the line is not a directive.
    pub fn detect_from(line: &str) -> Option<Directive> {
        // Get leading whitespaces
        let first_non_whitespace = line
            .find(|c: char| !c.is_whitespace())
            .unwrap_or(line.len());
        let whitespaces = &line[..first_non_whitespace];
        let line = &line[first_non_whitespace..];

        // Get prefix
        let (line, prefix) = match line.find(TXTPP_HASH) {
            Some(i) => (&line[i..], &line[..i]),
            None => return None,
        };

        // Get directive name and first argument
        let directive_name = &line[TXTPP_HASH.len()..];
        let (directive_name, arg) = match directive_name.split_once(' ') {
            Some((directive_name, arg)) => (directive_name, arg.trim_matches(char::is_whitespace)),
            None => (directive_name, ""),
        };

        // Parse type
        let diretive_type = match DirectiveType::try_from(directive_name) {
            Ok(x) => x,
            Err(_) => return None,
        };

        Some(Directive::new(
            whitespaces,
            prefix,
            diretive_type,
            vec![arg.to_string()],
        ))
    }
}

#[cfg(test)]
mod ut {
    use std::vec;

    use super::*;

    #[test]
    fn test_from_basic_not_directive() {
        let line = "";
        let actual = Directive::detect_from(line);

        assert_eq!(None, actual);
    }

    #[test]
    fn test_from_basic_whitespace_not_directive() {
        let line = "  \t  \t ";
        let actual = Directive::detect_from(line);

        assert_eq!(None, actual);
    }

    #[test]
    fn test_from_basic_randomstuff_not_directive() {
        let line = "  random  stuff\t\t";
        let actual = Directive::detect_from(line);

        assert_eq!(None, actual);
    }

    #[test]
    fn test_from_basic_empty() {
        let line = "TXTPP#";
        let expected = Some(Directive::new(
            "",
            "",
            DirectiveType::Empty,
            vec!["".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_empty_arg() {
        let line = "TXTPP# \t\t argag";
        let expected = Some(Directive::new(
            "",
            "",
            DirectiveType::Empty,
            vec!["argag".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_invalid() {
        let line = "TXTPP#nonext";
        let actual = Directive::detect_from(line);

        assert_eq!(None, actual);
    }

    #[test]
    fn test_from_basic_invalid_randomstuff() {
        let line = "  random TXTPP#stuff\t\t \\";
        let actual = Directive::detect_from(line);

        assert_eq!(None, actual);
    }

    #[test]
    fn test_from_basic_empty_randomstuff() {
        let line = "  random TXTPP# stuff\t\t";
        let expected = Some(Directive::new(
            "  ",
            "random ",
            DirectiveType::Empty,
            vec!["stuff".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_include_noarg() {
        let line = "TXTPP#include";
        let expected = Some(Directive::new(
            "",
            "",
            DirectiveType::Include,
            vec!["".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_include_noarg_space() {
        let line = "TXTPP#include ";
        let expected = Some(Directive::new(
            "",
            "",
            DirectiveType::Include,
            vec!["".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_include_noarg_many_spaces() {
        let line = "TXTPP#include  \t \t   ";
        let expected = Some(Directive::new(
            "",
            "",
            DirectiveType::Include,
            vec!["".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_include_one_space() {
        let line = "TXTPP#include hello";
        let expected = Some(Directive::new(
            "",
            "",
            DirectiveType::Include,
            vec!["hello".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_include_many_spaces() {
        let line = "TXTPP#include   \t \t hello";
        let expected = Some(Directive::new(
            "",
            "",
            DirectiveType::Include,
            vec!["hello".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_include_trailing_spaces() {
        let line = "TXTPP#include   \t \t hello \t \t  ";
        let expected = Some(Directive::new(
            "",
            "",
            DirectiveType::Include,
            vec!["hello".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_run_noarg() {
        let line = "TXTPP#run";
        let expected = Some(Directive::new(
            "",
            "",
            DirectiveType::Run,
            vec!["".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_run_noarg_space() {
        let line = "TXTPP#run ";
        let expected = Some(Directive::new(
            "",
            "",
            DirectiveType::Run,
            vec!["".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_run_noarg_many_spaces() {
        let line = "TXTPP#run  \t \t   ";
        let expected = Some(Directive::new(
            "",
            "",
            DirectiveType::Run,
            vec!["".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_run_one_space() {
        let line = "TXTPP#run hello";
        let expected = Some(Directive::new(
            "",
            "",
            DirectiveType::Run,
            vec!["hello".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_starting_whitespaces() {
        let line = "  \t  \t TXTPP#include hello ";
        let expected = Some(Directive::new(
            "  \t  \t ",
            "",
            DirectiveType::Include,
            vec!["hello".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }
    #[test]
    fn test_from_starting_chars() {
        let line = "abcdefgTXTPP#include hello ";
        let expected = Some(Directive::new(
            "",
            "abcdefg",
            DirectiveType::Include,
            vec!["hello".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_starting_chars_whitespace() {
        let line = "abcdefg  \tTXTPP#include hello ";
        let expected = Some(Directive::new(
            "",
            "abcdefg  \t",
            DirectiveType::Include,
            vec!["hello".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_starting_whitespace_chars() {
        let line = "  \t  abcdefgTXTPP#include hellow  hellw ";
        let expected = Some(Directive::new(
            "  \t  ",
            "abcdefg",
            DirectiveType::Include,
            vec!["hellow  hellw".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_starting_whitespace_chars_shitespace() {
        let line = "\t\t  <!-- TXTPP#include   \thellow  hellw ";
        let expected = Some(Directive::new(
            "\t\t  ",
            "<!-- ",
            DirectiveType::Include,
            vec!["hellow  hellw".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_detect_temp() {
        let line = "  random TXTPP#temp stuff\t\t";
        let expected = Some(Directive::new(
            "  ",
            "random ",
            DirectiveType::Temp,
            vec!["stuff".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_detect_tag() {
        let line = "  random TXTPP#tag stuff\t\t";
        let expected = Some(Directive::new(
            "  ",
            "random ",
            DirectiveType::Tag,
            vec!["stuff".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_detect_write() {
        let line = "  random TXTPP#write stuff\t\t";
        let expected = Some(Directive::new(
            "  ",
            "random ",
            DirectiveType::Write,
            vec!["stuff".to_string()],
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }
}
