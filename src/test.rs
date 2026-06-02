#[cfg(test)]
mod test {
	use axum::http::HeaderMap;
	use axum::http::HeaderName;
	use reqwest::IntoUrl;
	use serde::Serialize;
	use serde::Deserialize;
	use serde::de::DeserializeOwned;
	use std::sync::LazyLock;

	static TOKEN: &'static str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJsb2NhbGhvc3QiLCJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsInNjb3BlcyI6WyJvcGVuaWQiLCJlbWFpbCIsInByb2ZpbGUiXX0.LhTFN4JUE0PMOiMSi4bbzncHXxNN3Z0UpFCyUeIL1O4hHwoFHpnWjJMAJf1oWWTaZIOsl1pMhaWJ6SswoAS7bTvH8sCXabGpvzkboy7so--Nk5bFR8GMOpo3zKcsvva6Xr97wYKsINt9TXzcCeDCabYHRHITfxTTyp73dlSC-XeGzqTT6Ou8BU9pRpQIG0i5SfROi311WxbWfCpGofdUv8EzK5BsAgrXNlTjiFcv1eYM5f4EAWC_WIz81HfdMZhbEZAf1ELDjF0XXrZmzzeVFgQsUO7zZ7eszH_c10KHxooZCHBQoQtrITbd5eIw5Ki296llLSCC_f5Qo_U7a4XyEg";
	static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| reqwest::Client::builder()
		.build()
		.expect("Failed to initialise client"));

	#[tokio::main]
	#[test]
	async fn test_token() -> Result<(), reqwest::Error> {
		let res = get::<serde_json::Value>("http://localhost:1920/uuid/info/res").await?;
		println!("Res: {res:#?}");

		Ok(())
	}

	async fn get<Res: DeserializeOwned>(url: impl IntoUrl) -> reqwest::Result<Res> {
		return CLIENT.get(url)
			.bearer_auth(TOKEN)
			.send()
			.await?
			.json()
			.await;
	}

	async fn post<Res: DeserializeOwned>(url: impl IntoUrl, body: impl Serialize) -> reqwest::Result<Res> {
		return CLIENT.post(url)
			.bearer_auth(TOKEN)
			.json(&body)
			.send()
			.await?
			.json()
			.await;
	}
}
