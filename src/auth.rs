use crate::http_request::http_get;
use crate::State;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use jsonwebtoken::jwk::PublicKeyUse;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::LazyLock;
use std::time::Instant;
use std::time::Duration;
use axum::Json;
use axum::response::IntoResponse;
use serde_json::json;
use tokio::sync::RwLock;

structstruck::strike! {
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct TokenData {
		pub sub: String,
	}
}

static OIDC: OIDCDiscoveryManager = OIDCDiscoveryManager::new();

/// Middleware die den Auth Token ausliest, auswertet und nützliche Information in die Anfrage einfügt.
pub async fn auth_jwt(config: State, mut req: Request<Body>, next: axum::middleware::Next) -> axum::response::Response<Body> {
	// Token aus dem `Authorization` header auslesen
	let Some(token) = req.headers().get("Authorization").and_then(parse_header) else {
		return (StatusCode::UNAUTHORIZED, Json(json! {{
			"success": false,
			"error": "Authorization header was not provided or is not properly formatted.",
		}})).into_response();
	};

	let oidc = OIDC.get(&config.args.oidc).await;
	let jwks = match get_jwk(oidc.jwks_uri).await {
		Ok(jwks) => jwks,
		Err(err) => {
			log::error!("Could not get jwks: {err:?}");
			return (StatusCode::INTERNAL_SERVER_ERROR, Json(json! {{
				"success": false,
				"error": "Authorization was unable to proceed because the trust between the identity provider could not be established."
			}})).into_response();
		}
	};

	let mut validation = jsonwebtoken::Validation::new(Algorithm::RS256);

	validation.set_audience(&[&config.args.audience]);
	validation.set_issuer(&[&oidc.issuer]);

	validation.validate_exp = false;

	let Some(jsonwebtoken::TokenData { claims: token, .. }) = validate_token(token, validation, jwks) else {
		return (StatusCode::UNAUTHORIZED, Json(json! {{
			"success": false,
			"error": "Token was rejected."
		}})).into_response();
	};

	log::debug!("Der benutzer darf: {token:#?}");

	req.extensions_mut().insert(token);
	next.run(req).await
}

fn parse_header(header: &axum::http::HeaderValue) -> Option<&str> {
	header
		.to_str()
		.ok()?
		.strip_prefix("Bearer ")
		.map(|s| s.trim())
}


// JWK sind die Web-Keys Daten. Dort sind Zertifikate usw beinhaltet, den Clients ermöglicht die Validität des IdPs zu bestätigen, und somit die Integrität der Tokens zu garantieren.
/// Leider ist nicht definierbar wie lange die Zertifikate gültig sind. Das müsste man aus den Zertifikaten selber herauslesen.
/// Da das aber ein Haufen Aufwand ist, und deutlich weniger aufwand sie einfach stündlich zu aktualisieren, machen wir das einfach so.
static JWK: LazyLock<tokio::sync::RwLock<Option<JwkSet>>> =
	LazyLock::new(|| tokio::sync::RwLock::new(None));

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OIDCDiscovery {
	token_endpoint: url::Url,
	issuer: String,
	jwks_uri: url::Url,
}

async fn get_jwk(oidc: url::Url) -> Result<JwkSet, reqwest::Error> {
	if let Some(jwk) = JWK.read().await.as_ref() {
		return Ok(jwk.clone());
	}

	let oidc = OIDC.get(&oidc).await;
	let jwks: JwkSet = http_get(oidc.jwks_uri.clone()).await?;

	log::trace!("Updated JWKs: {jwks:#?}");

	*JWK.write().await = Some(jwks.clone());

	Ok(jwks)
}

const REFRESH_OIDC_DISCOVERY_EVERY: Duration = Duration::from_hours(1);

struct OIDCDiscoveryManager {
	oidc: LazyLock<RwLock<Option<(OIDCDiscovery, Instant)>>>,
}

impl OIDCDiscoveryManager {
	const fn new() -> Self {
		Self {
			oidc: LazyLock::new(|| RwLock::new(None)),
		}
	}

	/// Der OIDC Discovery Endpunkt ist eine JSON Ressource die Informationen über den OIDC Tokenaustausch, die Authorität usw beinhaltet.
	/// Wir können diese auslesen um einen großteil der Daten die wir benötigen zu bekommen. Der Rest muss man leider abfragen.
	/// Diese ändert sich recht selten, daher können wir sie auch zwischenspeichern.
	async fn get(&self, url: &::url::Url) -> OIDCDiscovery {
		if let Some((oidc, expires)) = self.oidc.read().await.as_ref()
			&& expires.elapsed() < REFRESH_OIDC_DISCOVERY_EVERY
		{
			return oidc.clone();
		}

		let data = http_get::<OIDCDiscovery>(url.clone())
			.await
			.expect("Failed to refresh OIDC discovery data.");
		self.oidc
			.write()
			.await
			.replace((data.clone(), Instant::now()));

		log::debug!("OIDC discovered: {data:#?}");

		data
	}
}

fn validate_token(token: impl AsRef<str>, validation: jsonwebtoken::Validation, jwks: JwkSet) -> Option<jsonwebtoken::TokenData<TokenData>> {
	for key in jwks.keys.iter() {
		if let Some(PublicKeyUse::Signature) = key.common.public_key_use {
			let key = match DecodingKey::from_jwk(key) {
				Ok(key) => key,
				Err(err) => {
					log::trace!("Failing key: {err:?}");
					continue;
				}
			};

			match jsonwebtoken::decode(token.as_ref(), &key, &validation) {
				Ok(token) => return Some(token),
				Err(err) => {
					log::trace!("Failing token: {err:?}");
					continue;
				}
			}
		} else {
			continue
		}
	}

	None
}
