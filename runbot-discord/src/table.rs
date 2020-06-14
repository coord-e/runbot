use std::fmt;

use runbot::action;
use runbot::model::compiler::Compiler;
use runbot::model::language::Language;

use itertools::Itertools;
use tabular::Row;

pub struct Table<T>(pub T);

impl fmt::Display for Table<Vec<Compiler>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut table = tabular::Table::new("{:<}  {:<}");
        for c in &self.0 {
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

impl fmt::Display for Table<Vec<Language>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut table = tabular::Table::new("{:<}  {:<}");
        for l in &self.0 {
            table.add_row(
                Row::new()
                    .with_cell(l.name())
                    .with_cell(l.aliases().iter().join(",")),
            );
        }
        write!(f, "{}", table)
    }
}

impl fmt::Display for Table<action::dump_setting::Output> {
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
