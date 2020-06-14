use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Response(pub Vec<Compiler>);

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Compiler {
    pub compiler_option_raw: bool,
    pub runtime_option_raw: bool,
    pub display_compile_command: String,
    pub switches: Vec<Switch>,
    pub name: String,
    pub version: String,
    pub language: String,
    pub display_name: String,
    pub templates: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Switch {
    #[serde(rename_all = "kebab-case")]
    Single {
        default: bool,
        name: String,
        display_flags: String,
        display_name: String,
    },
    Select {
        default: String,
        options: Vec<SwitchOption>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct SwitchOption {
    pub name: String,
    pub display_flags: String,
    pub display_name: String,
}
