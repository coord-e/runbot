use std::str::{pattern::Pattern, FromStr};

use runbot::model::code::Code;

use err_derive::Error;

#[derive(Clone, Debug)]
pub struct CodeBlock {
    language: Option<String>,
    code: String,
}

trait TakeWhile {
    fn take_while<'a, P>(&'a self, pat: P) -> Option<(&'a Self, &'a Self)>
    where
        P: Pattern<'a>;
}

impl TakeWhile for str {
    fn take_while<'a, P>(&'a self, pat: P) -> Option<(&'a Self, &'a Self)>
    where
        P: Pattern<'a>,
    {
        let split = self.splitn(2, pat).collect::<Vec<_>>();
        match &split[..] {
            [head, rest] => Some((head, rest)),
            _ => None,
        }
    }
}

fn parse_code_block(s: &str) -> Option<(CodeBlock, &str)> {
    let rest = s.trim();
    let rest = match rest.strip_prefix("```") {
        Some(x) => x,
        None => return None,
    };
    let (language, rest) = match rest.take_while('\n') {
        Some((head, rest)) if !head.is_empty() && !head.contains(' ') => (Some(head), rest),
        _ => (None, rest),
    };
    let (code, rest) = match rest.take_while("```") {
        Some(x) => x,
        None => return None,
    };
    Some((
        CodeBlock {
            language: language.map(str::to_owned),
            code: code.to_owned(),
        },
        rest,
    ))
}

impl CodeBlock {
    fn into_code(self) -> Code {
        match self {
            CodeBlock {
                code,
                language: Some(l),
            } => Code::with_language(code, l.parse().into_ok()),
            CodeBlock {
                code,
                language: None,
            } => Code::without_language(code),
        }
    }

    fn is_stdin(&self) -> bool {
        match self.language.as_deref() {
            Some("stdin") => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct CodeInput {
    code: CodeBlock,
    stdin: Option<String>,
}

#[derive(Debug, Error)]
pub enum ParseCodeInputError {
    #[error(display = "no code block found in the input")]
    NoCodeBlockFound,
    #[error(display = "only stdin block found in the input")]
    OnlyStdinFound,
    #[error(display = "too many code blocks in the input")]
    TooManyCodeBlocks,
}

impl FromStr for CodeInput {
    type Err = ParseCodeInputError;
    fn from_str(s: &str) -> Result<CodeInput, ParseCodeInputError> {
        let mut blocks = itertools::unfold(s, |input| {
            if let Some((block, rest)) = parse_code_block(input) {
                *input = rest;
                Some(block)
            } else {
                None
            }
        });

        match (blocks.next(), blocks.next(), blocks.next()) {
            (None, _, _) => Err(ParseCodeInputError::NoCodeBlockFound),
            (Some(code), None, _) if code.is_stdin() => Err(ParseCodeInputError::OnlyStdinFound),
            (Some(code), None, _) => Ok(CodeInput { code, stdin: None }),
            (Some(code1), Some(code2), None) if code1.is_stdin() => Ok(CodeInput {
                code: code2,
                stdin: Some(code1.code),
            }),
            (Some(code1), Some(code2), None) if code2.is_stdin() => Ok(CodeInput {
                code: code1,
                stdin: Some(code2.code),
            }),
            (Some(_), Some(_), _) => Err(ParseCodeInputError::TooManyCodeBlocks),
        }
    }
}

impl CodeInput {
    pub fn into_code(self) -> Code {
        self.code.into_code()
    }

    pub fn stdin(&self) -> Option<&String> {
        self.stdin.as_ref()
    }
}
