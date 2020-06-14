use crate::api;
use crate::Result;

use url::Url;

pub struct Client {
    inner: reqwest::blocking::Client,
    api_home: Url,
}

impl Client {
    pub fn new(api_home: &str) -> Result<Client> {
        let inner = reqwest::blocking::Client::new();
        let api_home = Url::parse(api_home)?;
        Ok(Client { inner, api_home })
    }

    pub fn into_inner(self) -> reqwest::blocking::Client {
        self.inner
    }

    pub fn endpoint(&self, path: &str) -> Result<Url> {
        self.api_home.join(path).map_err(Into::into)
    }

    pub fn compile(&self, request: &api::compile::Request) -> Result<api::compile::Response> {
        let endpoint = self.endpoint("compile.json")?;
        let body = serde_json::to_vec(request)?;

        use reqwest::header::CONTENT_TYPE;
        let response_bytes = self
            .inner
            .post(endpoint)
            .body(body)
            .header(CONTENT_TYPE, "application/json")
            .send()?
            .bytes()?;

        let response = serde_json::from_slice(&response_bytes)?;
        Ok(response)
    }

    pub fn list(&self) -> Result<api::list::Response> {
        let endpoint = self.endpoint("list.json")?;
        let response_bytes = self.inner.get(endpoint).send()?.bytes()?;
        let response = serde_json::from_slice(&response_bytes)?;
        Ok(response)
    }
}
