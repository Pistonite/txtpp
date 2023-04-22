
mod directive_from;
mod directive_addln;
mod directive_fmt;
mod string;
pub use string::*;
mod tag;
pub use tag::*;
mod worker;
mod dependency;

mod execute;
pub use execute::*;

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
    /// The arguments of the directive
    pub args: Vec<String>,
}

impl Directive {
    /// Create a new directive
    pub fn new(whitespaces: &str, prefix: &str, directive_type: DirectiveType, args: Vec<String>) -> Self {
        Self {
            whitespaces: whitespaces.to_string(),
            prefix: prefix.to_string(),
            directive_type,
            args,
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
    /// Tag directive, argument is a string
    Tag,
    /// Temp directive, argument is file content
    Temp
}

impl TryFrom<&str> for DirectiveType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "" => Ok(DirectiveType::Empty),
            "include" => Ok(DirectiveType::Include),
            "run" => Ok(DirectiveType::Run),
            "tag" => Ok(DirectiveType::Tag),
            "temp" => Ok(DirectiveType::Temp),
            _ => Err(()),
        }
    }
}
