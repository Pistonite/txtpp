
mod directive_from;
mod directive_addln;
mod directive_fmt;

/// Directive struct
///
/// This object stores information detected from line that contains a directive
#[derive(Debug, PartialEq)]
pub struct Directive {
    /// The whitespaces string before the first non-whitespace character in the line
    pub whitespaces: String,
    /// The content from the first non-whitespace to the directive prefix `TXTPP#`
    pub prefix: String,
    /// The directive type
    pub directive_type: DirectiveType,
    /// The argument of the directive
    pub arg: String,
}

impl Directive {
    /// Create a new directive
    pub fn new(whitespaces: &str, prefix: &str, directive_type: DirectiveType, arg: &str) -> Self {
        Self {
            whitespaces: whitespaces.to_string(),
            prefix: prefix.to_string(),
            directive_type,
            arg: arg.to_string(),
        }
    }
}


/// Directive type
#[derive(Debug, PartialEq)]
pub enum DirectiveType {
    /// Empty directive
    Empty,
    /// Include directive, argument is path to a file
    Include,
    /// Run directive, argument is a command
    Run,
}

/// Result of parsing directive
#[derive(Debug, PartialEq)]
pub enum Result {
    /// The line is not a directive
    NotDirective,
    /// The line is a directive
    Directive(Directive),
    /// The line is a directive, but need more lines to get the directive
    NeedNextLine(Directive),
}

impl Result {
    /// Convert the result to a directive
    /// 
    /// Panics if the result is [`Result::NotDirective`]
    #[inline]
    pub fn unwrap(self) -> Directive {
        match self {
            Result::NotDirective => panic!("unwrap() called on Result::NotDirective"),
            Result::Directive(directive) => directive,
            Result::NeedNextLine(directive) => directive,
        }
    }

    /// Returns `true` if the result is [`Result::NotDirective`]
    #[inline]
    pub const fn is_none(&self) -> bool {
        match self {
            Result::NotDirective => true,
            _ => false,
        }
    }
}


