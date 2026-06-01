use std::cell::LazyCell;
use serde::de::DeserializeOwned;
use url::Url;

thread_local! {
    pub(crate) static CLIENT: LazyCell<reqwest::Client> = LazyCell::new(|| {
        reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Could not build insecure HTTP client")
    });
}

pub async fn http_get<Res: DeserializeOwned>(url: Url) -> Result<Res, reqwest::Error> {
    Ok(CLIENT.with(|client| client.get(reqwest::Url::from(url)))
        .send()
        .await?
        .json()
        .await?)
}
