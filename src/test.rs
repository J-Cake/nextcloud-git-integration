#[cfg(test)]
mod test {
	use reqwest::IntoUrl;
	use serde::Serialize;
	use serde::de::DeserializeOwned;
	use std::sync::LazyLock;

	static TOKEN: LazyLock<String> = LazyLock::new(|| std::env::var("TOKEN").expect("TOKEN required"));
	static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| reqwest::Client::builder()
		.build()
		.expect("Failed to initialise client"));

	#[tokio::main]
	#[test]
	async fn test_token() -> Result<(), reqwest::Error> {
		let res = get::<serde_json::Value>("http://git-server:1920/info/refs").await?;
		println!("Res: {res:#?}");

		assert_eq!(res.get("success").and_then(|val| val.as_bool()), Some(true));

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
