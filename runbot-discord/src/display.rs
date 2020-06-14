use std::fmt;

use runbot::action;
use runbot::model::compiler::Compiler;
use runbot::model::language::Language;

use super::compile_result::CompileResult;
use super::error::Error;

use itertools::Itertools;
use tabular::Row;

pub struct Display<'a, T>(pub &'a T);

impl fmt::Display for Display<'_, Vec<Compiler>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut table = tabular::Table::new("{:<}  {:<}");
        for c in self.0 {
            let version = if let Some(v) = c.version() {
                v.to_string()
            } else {
                "unknown".to_string()
            };
            table.add_row(Row::new().with_cell(c.name()).with_cell(version));
        }
        write!(f, "{}", table)
    }
}

impl fmt::Display for Display<'_, Vec<Language>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut table = tabular::Table::new("{:<}  {:<}");
        for l in self.0 {
            table.add_row(
                Row::new()
                    .with_cell(l.name())
                    .with_cell(l.aliases().iter().join(",")),
            );
        }
        write!(f, "{}", table)
    }
}

impl fmt::Display for Display<'_, action::dump_setting::Output> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut table = tabular::Table::new("{:<}  {:<}");
        table.add_row(Row::new().with_cell("auto").with_cell(self.0.auto));
        table.add_row(
            Row::new()
                .with_cell("auto-save")
                .with_cell(self.0.auto_save),
        );
        table.add_heading("remap:");
        for (l, c) in &self.0.remap {
            table.add_row(Row::new().with_cell(l).with_cell(c));
        }
        write!(f, "{}", table)
    }
}

impl fmt::Display for Display<'_, Error> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Error::Runbot(runbot::Error::UnknownLanguageName(name)) => {
                write!(f, "`{}` ？うーん...", name)
            }
            Error::Runbot(runbot::Error::UnknownCompilerName(name)) => {
                write!(f, "`{}` っ て 何 ？ 笑", name)
            }
            Error::Runbot(runbot::Error::UnknownCompilerSpec(name)) => write!(
                f,
                "`{}` とはなんですか？普通、`{}` とはならないとおもうのですが...",
                name, name
            ),
            Error::Runbot(runbot::Error::UnmappedLanguage(name)) => {
                write!(f, "`{}` に対応するコンパイラが決まっていない", name)
            }
            Error::Runbot(runbot::Error::NoCompilerSpecified) => {
                write!(f, "どのコンパイラを使えばいいかわかんないよ〜")
            }
            Error::Runbot(runbot::Error::RemapMismatch(c, l)) => {
                write!(f, "や、`{}` は `{}` でコンパイルできないよ", l, c)
            }
            Error::InvalidCodeInput(_) => write!(f, "コードの入力がおかしいよ"),
            Error::MalformedArguments(_) => write!(f, "ちょっと、いたずらしないでください"),
            Error::InvalidNumberOfArguments(n) => write!(f, "{}個の引数が必要だよ", n),
            Error::UnknownCommand(c) => write!(f, "`{}`、完全に理解した", c),
            Error::CommandIsMissing => write!(f, "？"),
            _ => write!(f, "ごめん"),
        }
    }
}

fn strip_ansi_escapes(s: impl AsRef<str>) -> Result<String, fmt::Error> {
    let s = strip_ansi_escapes::strip(s.as_ref()).map_err(|_| fmt::Error)?;
    String::from_utf8(s).map_err(|_| fmt::Error)
}

impl<T> fmt::Display for Display<'_, T>
where
    T: CompileResult,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(msg) = self.0.compiler_message() {
            let msg = strip_ansi_escapes(msg)?;
            writeln!(f, "```{}```", msg)?;
        }

        if let Some(msg) = self.0.program_message() {
            let msg = strip_ansi_escapes(msg)?;
            writeln!(f, "```{}```", msg)?;
        }

        if let Some(s) = self.0.signal() {
            writeln!(f, "exited with signal: {}", s)?;
        }

        if let Some(s) = self.0.status() {
            writeln!(f, "exited with status code {}", s)?;
        }

        if let Some(s) = self.0.url() {
            writeln!(f, "{}", s)?;
        }

        Ok(())
    }
}
