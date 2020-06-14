use crate::model::code::Code;
use crate::{Context, Error, Result};

pub enum Output {
    NoRun,
    Run {
        status: Option<u32>,
        signal: Option<String>,
        compiler_message: Option<String>,
        program_message: Option<String>,
        url: Option<String>,
    },
}

pub fn run_implicit(ctx: &Context, code: Code, stdin: Option<String>) -> Result<Output> {
    if !ctx.is_auto()? {
        return Ok(Output::NoRun);
    }

    let save = ctx.is_auto_save()?;

    let compiler = if let Some(lang) = code.language() {
        ctx.resolve_language_name(lang)?
    } else {
        return Err(Error::NoCompilerSpecified);
    };

    let req = wandbox::api::compile::Request {
        compiler: compiler.wandbox_name().clone(),
        code: code.text().clone(),
        codes: Vec::new(),
        options: None,
        stdin,
        compiler_option_raw: None,
        runtime_option_raw: None,
        save,
    };

    let res = ctx.wandbox.compile(&req)?;
    Ok(Output::Run {
        status: res.status.map(|o| o.parse().unwrap()),
        signal: res.signal,
        compiler_message: res.compiler_message,
        program_message: res.program_message,
        url: res.url,
    })
}
