use crate::constants::TXTPP_HASH;
use super::*;

impl Directive {

    /// Try detecting a directive from a line
    /// 
    /// The line should not have new line characters (`\r\n` or `\n`) at the end
    pub fn detect_from(line: &str) -> Result {
        let first_non_whitespace = line
            .find(|c: char| !c.is_whitespace())
            .unwrap_or(line.len());
        let whitespaces = &line[..first_non_whitespace];
        let line = &line[first_non_whitespace..];
    
        let (line, prefix) = match line.find(TXTPP_HASH) {
            Some(i) => (
                &line[i..],
                &line[..i],
            ),
            None => return Result::NotDirective,
        };
    
        let directive_name = &line[TXTPP_HASH.len()..];
        let (directive_name, arg) = match directive_name.split_once(' ') {
            Some((directive_name, arg)) => (directive_name, arg),
            None => (directive_name, ""),
        };
        let (need_more_lines, arg) = if arg.ends_with('\\') {
            (true, arg[..arg.len() - 1].trim_start_matches(char::is_whitespace))
        } else {
            (false, arg.trim_matches(char::is_whitespace))
        };
        
        let directive = match directive_name {
            "" => Directive::new(
                whitespaces,
                prefix,
                DirectiveType::Empty,
                "", // we don't need the arg in empty directive
            ),
            "include" => {
                if arg.is_empty() {
                    return Result::NotDirective;
                } else {
                    Directive::new(
                        whitespaces,
                        prefix,
                        DirectiveType::Include,
                        arg,
                    )
                }
            }
            "run" => {
                if arg.is_empty() {
                    return Result::NotDirective;
                } else {
                    Directive::new(
                        whitespaces,
                        prefix,
                        DirectiveType::Run,
                        arg,
                    )
                }
            }
            _ => return Result::NotDirective,
        };
    
        if need_more_lines {
            Result::NeedNextLine(directive)
        } else {
            Result::Directive(directive)
        }
    }
    
}

#[cfg(test)]
mod ut {
    use super::*;

    #[test]
    fn test_from_basic_not_directive() {
        let line = "";
        let expected = Result::NotDirective;
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_whitespace_not_directive() {
        let line = "  \t  \t ";
        let expected = Result::NotDirective;
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_randomstuff_not_directive() {
        let line = "  random  stuff\t\t";
        let expected = Result::NotDirective;
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_randomstuff_not_directive_multiline() {
        let line = "  random  stuff\t\t \\";
        let expected = Result::NotDirective;
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_empty() {
        let line = "TXTPP#";
        let expected = Result::Directive(Directive::new("", "", DirectiveType::Empty, ""));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_empty_arg() {
        let line = "TXTPP# \t\t argag";
        let expected = Result::Directive(Directive::new("", "", DirectiveType::Empty, ""));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_empty_multiline() {
        let line = "TXTPP# argag \\";
        let expected = Result::NeedNextLine(Directive::new("", "", DirectiveType::Empty, ""));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_invalid() {
        let line = "TXTPP#nonext";
        let expected =
            Result::NotDirective;
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_invalid_multiline() {
        let line = "TXTPP#nonext asdfasdf \\";
        let expected =
            Result::NotDirective;
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_invalid_randomstuff() {
        let line = "  random TXTPP#stuff\t\t \\";
        let expected = Result::NotDirective;
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_empty_randomstuff() {
        let line = "  random TXTPP# stuff\t\t \\";
        let expected = Result::NeedNextLine(Directive::new("  ", "random ", DirectiveType::Empty, ""));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_include_noarg() {
        let line = "TXTPP#include";
        let expected = Result::NotDirective;
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_include_noarg_space() {
        let line = "TXTPP#include ";
        let expected = Result::NotDirective;
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_include_noarg_many_spaces() {
        let line = "TXTPP#include  \t \t   ";
        let expected = Result::NotDirective;
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_include_one_space() {
        let line = "TXTPP#include hello";
        let expected =
            Result::Directive(Directive::new("", "", DirectiveType::Include, "hello"));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_include_many_spaces() {
        let line = "TXTPP#include   \t \t hello";
        let expected =
            Result::Directive(Directive::new("", "", DirectiveType::Include, "hello"));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_include_trailing_spaces() {
        let line = "TXTPP#include   \t \t hello \t \t  ";
        let expected =
            Result::Directive(Directive::new("", "", DirectiveType::Include, "hello"));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_run_noarg() {
        let line = "TXTPP#run";
        let expected = Result::NotDirective;
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_run_noarg_space() {
        let line = "TXTPP#run ";
        let expected = Result::NotDirective;
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_run_noarg_many_spaces() {
        let line = "TXTPP#run  \t \t   ";
        let expected = Result::NotDirective;
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_run_one_space() {
        let line = "TXTPP#run hello";
        let expected =
            Result::Directive(Directive::new("", "", DirectiveType::Run, "hello"));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_run_many_spaces() {
        let line = "TXTPP#run   \t \t hello";
        let expected =
            Result::Directive(Directive::new("", "", DirectiveType::Run, "hello"));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_basic_run_trailing_spaces() {
        let line = "TXTPP#run   \t \t hello \t \t  ";
        let expected =
            Result::Directive(Directive::new("", "", DirectiveType::Run, "hello"));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_starting_whitespaces() {
        let line = "  \t  \t TXTPP#include hello ";
        let expected = Result::Directive(Directive::new(
            "  \t  \t ",
            "",
            DirectiveType::Include,
            "hello",
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }
    #[test]
    fn test_from_starting_chars() {
        let line = "abcdefgTXTPP#include hello ";
        let expected = Result::Directive(Directive::new(
            "",
            "abcdefg",
            DirectiveType::Include,
            "hello",
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_starting_chars_whitespace() {
        let line = "abcdefg  \tTXTPP#include hello ";
        let expected = Result::Directive(Directive::new(
            "",
            "abcdefg  \t",
            DirectiveType::Include,
            "hello",
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_starting_whitespace_chars() {
        let line = "  \t  abcdefgTXTPP#include hellow  hellw ";
        let expected = Result::Directive(Directive::new(
            "  \t  ",
            "abcdefg",
            DirectiveType::Include,
            "hellow  hellw",
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_starting_whitespace_chars_shitespace() {
        let line = "\t\t  <!-- TXTPP#include   \thellow  hellw ";
        let expected = Result::Directive(Directive::new(
            "\t\t  ",
            "<!-- ",
            DirectiveType::Include,
            "hellow  hellw",
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_multiline() {
        let line = "\t\t  <!-- TXTPP#include   \thellow  hellw \\";
        let expected = Result::NeedNextLine(Directive::new(
            "\t\t  ",
            "<!-- ",
            DirectiveType::Include,
            "hellow  hellw ",
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_multiline_trailing_whitespace() {
        let line = "\t\t  <!-- TXTPP#include   \thellow  hellw \\ ";
        let expected = Result::Directive(Directive::new(
            "\t\t  ",
            "<!-- ",
            DirectiveType::Include,
            "hellow  hellw \\",
        ));
        let actual = Directive::detect_from(line);

        assert_eq!(expected, actual);
    }

}
