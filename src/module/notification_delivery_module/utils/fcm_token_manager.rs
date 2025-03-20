use std::{
    env,
    path::Path,
    sync::{Arc, RwLock},
};

use gcp_auth::{CustomServiceAccount, Token, TokenProvider};
use log::error;

use crate::module::notification_delivery_module::errors::NotiDeliverError;

/// `TokenManager` handles Google Cloud authentication tokens used for FCM.
/// It fetches, stores, and updates the token when necessary
pub struct TokenManager {
    // Shared and thread-safe reference to the authentication token.
    token: Arc<RwLock<Arc<Token>>>,
}

impl TokenManager {
    /// Creates a new `TokenManager` instance and initializes the token.
    pub async fn new() -> Self {
        let token = Self::fetch_token()
            .await
            .unwrap_or_else(|e| panic!("{}", e));

        let shared_token = Arc::new(RwLock::new(token));

        Self {
            token: shared_token,
        }
    }

    /// Fetches a new authentication token from Google Cloud.
    async fn fetch_token() -> Result<Arc<Token>, NotiDeliverError> {
        // Read the path to the Google credentials JSON file
        let credential_path = env::var("GOOGLE_APPLICATION_CREDENTIALS").map_err(|e| {
            error!("Missing GOOGLE_APPLICATION_CREDENTIALS: {}", e);
            NotiDeliverError::MissingEnvError(e)
        })?;

        // Load service account credentials from the file
        let service_account = CustomServiceAccount::from_file(Path::new(&credential_path))
            .map_err(|e| {
                error!("Cannot get gcp service account: {}", e);
                NotiDeliverError::GCPAuthError(e)
            })?;

        // Define required authentication scopes
        // This scope can be found on https://firebase.google.com/docs/cloud-messaging/auth-server#windows
        let scopes = &["https://www.googleapis.com/auth/firebase.messaging"];

        // Request an authentication token
        let token = service_account.token(scopes).await.map_err(|e| {
            error!("Cannot get gcp token: {}", e);
            NotiDeliverError::GCPAuthError(e)
        })?;

        Ok(token)
    }

    /// Retrieves the current authentication token
    pub fn get_token(&self) -> Option<Arc<Token>> {
        match self.token.read() {
            Ok(token) => Some(token.clone()),
            Err(_) => {
                error!("Can not read gcp token");
                None
            }
        }
    }

    /// Updates the stored authentication token by fetching a new one
    pub async fn update_token(&self) {
        match Self::fetch_token().await {
            Ok(new_token) => match self.token.write() {
                Ok(mut current_token) => {
                    *current_token = new_token;
                }
                Err(_) => {
                    error!("Cannot rewrite token");
                }
            },
            Err(_) => {
                error!("Update token failed")
            }
        }
    }
}
