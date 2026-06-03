use std::fs::exists;

use axum::Json;
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::extract::Path;
use axum::extract::Query;
use serde::Deserialize;

use crate::State;

#[derive(Debug, Deserialize)]
pub(crate) struct InfoRefsQuery {
	service: String,
}

pub(crate) async fn info(Path(repo): Path<String>, Query(q): Query<InfoRefsQuery>, config: State) -> impl IntoResponse {
	log::debug!("Repository: {repo}: {q:?}");

	let cmd = match q.service.as_str() {
		"git-upload-pack" => "upload-pack",
		"git-receive-pack" => "receive-pack",
		service => return (StatusCode::BAD_REQUEST, Json(serde_json::json! {{
			"success": false,
			"error": format!("Unrecognised service {service}")
		}})).into_response()
	};
}

pub(crate) async fn upload(Path(repo): Path<String>) -> impl IntoResponse {
	(StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json! {{
		"success": false,
		"error": "Method not implemented yet."
	}})).into_response()
}

pub(crate) async fn receive(Path(repo): Path<String>) -> impl IntoResponse {
	(StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json! {{
		"success": false,
		"error": "Method not implemented yet."
	}})).into_response()
}
