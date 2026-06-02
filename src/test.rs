#[cfg(test)]
mod test {
	use axum::http::HeaderMap;
	use axum::http::HeaderName;
	use reqwest::IntoUrl;
	use serde::Serialize;
	use serde::Deserialize;
	use serde::de::DeserializeOwned;
	use std::sync::LazyLock;

	static TOKEN: LazyLock<String> = LazyLock::new(|| std::env::var("TOKEN").expect("TOKEN required"));
	static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| reqwest::Client::builder()
		.build()
		.expect("Failed to initialise client"));

	#[tokio::main]
	#[test]
	async fn test_token() -> Result<(), reqwest::Error> {
		let res = get::<serde_json::Value>("http://localhost:1920/uuid/info/refs").await?;
		println!("Res: {res:#?}");

		Ok(())
	}

	async fn get<Res: DeserializeOwned>(url: impl IntoUrl) -> reqwest::Result<Res> {
		return CLIENT.get(url)
			.bearer_auth(TOKEN.as_str())
			.send()
			.await?
			.json()
			.await;
	}

	async fn post<Res: DeserializeOwned>(url: impl IntoUrl, body: impl Serialize) -> reqwest::Result<Res> {
		return CLIENT.post(url)
			.bearer_auth(TOKEN.as_str())
			.json(&body)
			.send()
			.await?
			.json()
			.await;
	}
}
