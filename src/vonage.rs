use std::{env::VarError, time::Duration};

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::app_error::AppError;

const VERIFY_ENDPOINT: &str = "https://api.nexmo.com/v2/verify";
const EU_ENDPOINT: &str = "https://api-eu.vonage.com/identity-insights/v1/requests";
const US_ENDPOINT: &str = "https://api-us.vonage.com/identity-insights/v1/requests";
const FRAUD_PREVENTION_PURPOSE: &str = "FraudPreventionAndDetection";
/// The SIM Swap API's lookback period, in hours.
const SIM_SWAP_PERIOD_HOURS: u16 = 24 * 7;

/// Shared HTTP client and Vonage API operations.
#[derive(Clone)]
pub struct VonageClient {
    http_client: reqwest::Client,
}

#[derive(Serialize)]
struct IdentityInsightsRequest {
    phone_number: String,
    purpose: &'static str,
    insights: InsightsRequest,
}

#[derive(Serialize)]
struct InsightsRequest {
    sim_swap: SimSwapRequest,
}

#[derive(Serialize)]
struct SimSwapRequest {
    period: u16,
}

#[derive(Deserialize)]
struct IdentityInsightsResponse {
    insights: InsightsResponse,
}

#[derive(Deserialize)]
struct InsightsResponse {
    sim_swap: SimSwapResponse,
}

#[derive(Deserialize)]
struct SimSwapResponse {
    is_swapped: Option<bool>,
    status: InsightStatus,
}

#[derive(Deserialize)]
struct InsightStatus {
    code: String,
}

// Verify V2
#[derive(Serialize)]
struct VerifyWorkflow {
    channel: &'static str,
    to: String,
}

#[derive(Serialize)]
struct VerifyRequest {
    brand: &'static str,
    workflow: Vec<VerifyWorkflow>,
    channel_timeout: u16,
}

#[derive(Deserialize)]
struct VerifyResponse {
    request_id: String,
}

#[derive(Serialize)]
struct CheckCodeRequest {
    code: String,
}

impl VonageClient {
    /// Create a Vonage client with a bounded request timeout.
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Unable to create Vonage HTTP client");
        Self { http_client }
    }

    /// Check whether a phone number has had a recent SIM swap.
    ///
    /// This function fails closed: no OTP is sent when the insight reports a
    /// recent swap, an unsupported number, or an unavailable provider result.
    pub async fn check_sim_swap(
        &self,
        vonage_token: &str,
        phone_number: &str,
    ) -> Result<(), AppError> {
        let request = IdentityInsightsRequest {
            // phone_number: "990123455".to_owned(),
            phone_number: phone_number.to_owned(),
            purpose: FRAUD_PREVENTION_PURPOSE,
            insights: InsightsRequest {
                sim_swap: SimSwapRequest {
                    period: SIM_SWAP_PERIOD_HOURS,
                },
            },
        };
        let response = self
            .http_client
            .post(identity_insights_endpoint()?)
            .bearer_auth(vonage_token)
            .json(&request)
            .send()
            .await?;
        if !response.status().is_success() {
            warn!(status = %response.status(), "Identity Insights rejected SIM swap request");
            return Err(AppError::ExternalApiError(
                "Phone risk check was unavailable".to_string(),
            ));
        }
        let data: IdentityInsightsResponse = response.json().await.map_err(|error| {
            warn!(error = %error, "Identity Insights returned an invalid response body");
            AppError::ExternalApiError("Phone risk check returned an invalid response".to_string())
        })?;
        let sim_swap = data.insights.sim_swap;

        if sim_swap.status.code != "OK" {
            warn!(
                status_code = %sim_swap.status.code,
                "Identity Insights returned an unsuccessful SIM swap result"


            );
            return Err(AppError::ExternalApiError(
                "Phone risk check was unavailable".to_string(),
            ));
        }
        match sim_swap.is_swapped {
            Some(true) => Err(AppError::Forbidden(
                "Unable to send a verification code to this phone number".to_string(),
            )),
            Some(false) => Ok(()),
            None => {
                warn!("Identity Insights returned OK without an is_swapped result");
                Err(AppError::ExternalApiError(
                    "Phone risk check returned an incomplete response".to_string(),
                ))
            }
        }
    }

    /// Start a Verify request and return its request ID.
    pub async fn start_verification(
        &self,
        vonage_token: &str,
        phone_number: &str,
    ) -> Result<String, AppError> {
        let verify_body = VerifyRequest {
            brand: "Rock Solid",
            workflow: vec![VerifyWorkflow {
                channel: "sms",
                to: phone_number.to_owned(),
            }],
            channel_timeout: 600,
        };
        let response = self
            .http_client
            .post(VERIFY_ENDPOINT)
            .bearer_auth(vonage_token)
            .json(&verify_body)
            .send()
            .await?;
        if !response.status().is_success() {
            debug!("Verification provider rejected request with response: {response:?}");
            return Err(AppError::ExternalApiError(
                "Verification provider rejected the request".to_string(),
            ));
        }
        let verify_data: VerifyResponse = response.json().await?;
        debug!("REQUEST_ID=\"{}\"", verify_data.request_id);
        Ok(verify_data.request_id)
    }

    /// Check the verification code that a user has received.
    pub async fn validate_verification(
        &self,
        vonage_token: &str,
        request_id: &str,
        code: &str,
    ) -> Result<(), AppError> {
        let response = self
            .http_client
            .post(format!("{VERIFY_ENDPOINT}/{request_id}"))
            .bearer_auth(vonage_token)
            .json(&CheckCodeRequest {
                code: code.to_owned(),
            })
            .send()
            .await?;
        if !response.status().is_success() {
            debug!("Verification provider rejected validation request with response: {response:?}");
            return Err(AppError::ExternalApiError(
                "Verification provider rejected the validation".to_string(),
            ));
        }

        Ok(())
    }
}

/// Returns the Identity Insights endpoint. By default the European endpoint is returned, but it can be changed using
/// the environment variable `VONAGE_IDENTITY_INSIGHTS_REGION` to force the US one ("us") or the European one ("eu")
fn identity_insights_endpoint() -> Result<&'static str, AppError> {
    match std::env::var("VONAGE_IDENTITY_INSIGHTS_REGION").as_deref() {
        Ok("eu") | Err(VarError::NotPresent) => Ok(EU_ENDPOINT),
        Ok("us") => Ok(US_ENDPOINT),
        Ok(region) => Err(AppError::Internal(anyhow::anyhow!(
            "Unsupported VONAGE_IDENTITY_INSIGHTS_REGION: {region}"
        ))),
        Err(error) => Err(AppError::Internal(anyhow::anyhow!(
            "Unable to read VONAGE_IDENTITY_INSIGHTS_REGION: {error}"
        ))),
    }
}
