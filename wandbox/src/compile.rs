use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Request {
    pub compiler: String,
    pub code: String,
    pub codes: Vec<Code>,
    pub options: Option<String>,
    pub stdin: Option<String>,
    pub compiler_option_raw: Option<String>,
    pub runtime_option_raw: Option<String>,
    pub save: bool,
}

#[derive(Serialize, Debug)]
pub struct Code {
    pub file: String,
    pub code: String,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    pub status: Option<String>,
    pub signal: Option<String>,
    pub compiler_output: Option<String>,
    pub compiler_error: Option<String>,
    pub compiler_message: Option<String>,
    pub program_output: Option<String>,
    pub program_error: Option<String>,
    pub program_message: Option<String>,
    pub permlink: Option<String>,
    pub url: Option<String>,
}

pub fn compile(x: &Request) -> Result<Response, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    client
        .post("https://wandbox.org/api/compile.json")
        .json(x)
        .send()?
        .json()
}
