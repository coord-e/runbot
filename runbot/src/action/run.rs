use crate::model::code::Code;
use crate::model::compiler_options::CompilerOptions;
use crate::model::compiler_spec::CompilerSpec;
use crate::{ActionContext, Error, Result};

use itertools::Itertools;

pub struct Output {
    pub status: Option<u32>,
    pub signal: Option<String>,
    pub compiler_message: Option<String>,
    pub program_message: Option<String>,
    pub url: Option<String>,
}

// Notice that both `compiler_spec` and `code` can specify the compiler to use.
pub fn run(
    ctx: &ActionContext,
    compiler_spec: Option<CompilerSpec>,
    code: Code,
    options: Option<CompilerOptions>,
    stdin: Option<String>,
    save: bool,
) -> Result<Output> {
    let compiler = if let Some(spec) = compiler_spec {
        ctx.resolve_compiler_spec(&spec)?
    } else if let Some(lang) = code.language() {
        ctx.resolve_language_name(lang)?
    } else {
        return Err(Error::NoCompilerSpecified);
    };

    let req = wandbox::compile::Request {
        compiler: compiler.wandbox_name().clone(),
        code: code.text().clone(),
        codes: Vec::new(),
        options: None,
        stdin,
        compiler_option_raw: options.map(|o| o.into_iter().join("\n")),
        runtime_option_raw: None,
        save,
    };

    let res = wandbox::compile(&req)?;
    Ok(Output {
        status: res.status.map(|o| o.parse().unwrap()),
        signal: res.signal,
        compiler_message: res.compiler_message,
        program_message: res.program_message,
        url: res.url,
    })
}
