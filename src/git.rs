use std::fs::exists;

use axum::{Extension, Json};
use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::extract::Path;
use axum::extract::Query;
use serde::Deserialize;
use sqlx::testing::TestTermination;
use tokio::process::Command;
use crate::auth::TokenData;
use crate::State;

#[derive(Debug, Deserialize)]
pub(crate) struct InfoRefsQuery {
	service: String,
}

const UPLOAD: &'static str = "# service=git-upload-pack\n";
const RECEIVE: &'static str = "# service=git-receive-pack\n";

pub(crate) async fn info(Query(q): Query<InfoRefsQuery>, token: Extension<TokenData>, config: State) -> impl IntoResponse {
	let cmd = match q.service.as_str() {
		"git-upload-pack" => "upload-pack",
		"git-receive-pack" => "receive-pack",
		service => return (StatusCode::BAD_REQUEST, Json(serde_json::json! {{
			"success": false,
			"error": format!("Unrecognised service {service}")
		}})).into_response()
	};

	let path = config.args.root.join(&token.repo);

	if !tokio::fs::try_exists(&path).await.is_ok_and(|exists| exists) {
		return (StatusCode::NOT_FOUND, Json(serde_json::json! {{
			"success": false,
			"error": format!("Repository does not exist for {repo}", repo=token.repo)
		}})).into_response();
	}

	let mut out = match q.service.as_str() {
		"git-upload-pack" => format!("{:04x}{UPLOAD}0000", UPLOAD.len()),
		"git-receive-pack" => format!("{:04x}{RECEIVE}0000", RECEIVE.len()),
		_ => unreachable!()
	}.into_bytes();

	out.extend_from_slice(match Command::new("git")
		.args(&[cmd, "--stateless-rpc", "--advertise-refs"])
		.arg(&path)
		.output()
		.await {
		Ok(out) if out.status.success() => out.stdout,
		Ok(out) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json! {{
			"success": false,
			"error": format!("Git failed: {err}", err=String::from_utf8(out.stderr).unwrap_or_default())
		}})).into_response(),
		Err(err) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json! {{
			"success": false,
			"error": format!("Git failed: {err:?}")
		}})).into_response(),
	}.as_slice());

	([
		 (header::CONTENT_TYPE, format!("application/x-{service}-advertisement", service=&q.service)),
		 (header::CACHE_CONTROL, "no-cache".to_string()),
	], out).into_response()
}

pub(crate) async fn upload() -> impl IntoResponse {
	(StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json! {{
		"success": false,
		"error": "Method not implemented yet."
	}})).into_response()
}

pub(crate) async fn receive() -> impl IntoResponse {
	(StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json! {{
		"success": false,
		"error": "Method not implemented yet."
	}})).into_response()
}
