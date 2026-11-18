use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Request, State, rejection::JsonRejection},
    http::HeaderMap,
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{Level, debug, warn};

use crate::{
    app_error::AppError,
    repository::Repository,
    token::{
        get_decoding_key, get_encoding_key, produce_session_token, produce_vonage_token,
        verify_session_token,
    },
    user::User,
    vonage::VonageClient,
};

#[derive(Clone)]
struct AppState {
    repo: Arc<dyn Repository<User>>,
    vonage: VonageClient,
}

/// Require a valid application session before running a protected handler.
async fn require_session(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let decoding_key = get_decoding_key()?;
    verify_session_token(&decoding_key, &headers)?;
    Ok(next.run(request).await)
}

/// Return an HTTP router with the handlers bound to the endpoints.
pub fn app(repo: Arc<dyn Repository<User>>) -> Router {
    let state = AppState {
        repo,
        vonage: VonageClient::new(),
    };
    let protected_routes = Router::new()
        .route("/application", get(application))
        .route_layer(middleware::from_fn(require_session));

    Router::new()
        .route("/health", get(health))
        .route("/session", post(create_session))
        .route("/session/verification", post(validate_code))
        .merge(protected_routes)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        // .include_headers(true)
                        .level(Level::INFO),
                )
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(state)
}

#[derive(Serialize)]
struct HealthData {
    live: bool,
    ready: bool,
}

async fn health() -> Result<Json<HealthData>, AppError> {
    debug!("Health requested");
    Ok(Json(HealthData {
        live: true,
        ready: true,
    }))
}

#[derive(Deserialize)]
struct SessionRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct SessionResponse {
    /// Used to provide 2FA code.
    verify_request_id: String,
}

/// HTTP handler to create a session for a correspondent in the application.
async fn create_session(
    State(state): State<AppState>,
    payload: Result<Json<SessionRequest>, JsonRejection>,
) -> Result<Json<SessionResponse>, AppError> {
    let Json(payload) = payload?;
    let Some(user) = state.repo.find_by_id(&payload.username) else {
        warn!(username = payload.username, "User not found");
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    };
    debug!(username = payload.username, "Authenticating user");

    if !user.verify_password(&payload.password) {
        warn!(username = payload.username, "Wrong password for user");
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let vonage_token = produce_vonage_token()?;
    // Check the phone risk before creating a Verify request. This keeps the
    // OTP from being sent when a recent SIM swap is reported.
    state
        .vonage
        .check_sim_swap(&vonage_token, &user.phone)
        .await?;

    let verify_request_id = state
        .vonage
        .start_verification(&vonage_token, &user.phone)
        .await?;
    Ok(Json(SessionResponse { verify_request_id }))
}

/// `username` should never be accepted from the client. The previous step should store pending challenges with some
/// expiration deadline.
#[derive(Deserialize)]
struct ValidateCodeRequest {
    username: String, // WARN: This is not a secure implementation!!!
    request_id: String,
    code: String,
}

#[derive(Serialize)]
struct ValidateCodeResponse {
    token: String,
}

/// Complete 2FA for the session.
async fn validate_code(
    State(state): State<AppState>,
    payload: Result<Json<ValidateCodeRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<ValidateCodeResponse>), AppError> {
    let Json(payload) = payload?;

    let vonage_token = produce_vonage_token()?;
    state
        .vonage
        .validate_verification(&vonage_token, &payload.request_id, &payload.code)
        .await?;

    debug!(user = %payload.username, "Successful verification");
    let encoding_key = get_encoding_key()?;
    let token = produce_session_token(&encoding_key, &payload.username)?;

    Ok((StatusCode::CREATED, Json(ValidateCodeResponse { token })))
}

#[derive(Serialize)]
struct ApplicationData {
    message: &'static str,
}

/// Example endpoint protected by the session middleware.
async fn application() -> Result<Json<ApplicationData>, AppError> {
    Ok(Json(ApplicationData {
        message: "Application data",
    }))
}
