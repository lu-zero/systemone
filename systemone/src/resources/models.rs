use isahc::http::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::error::Error;

/// Metadata for an available model.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelCard {
    pub name: String,
    pub description: String,
    pub release_date: String,
}

/// Response body for `GET /v1/models`.
#[derive(Debug, Deserialize)]
struct ModelsWire {
    models: Vec<ModelCard>,
}

/// Access to the models API resource, borrowed from [`Client::models`].
pub struct Models<'a> {
    client: &'a Client,
}

impl<'a> Models<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List the models available to the account.
    pub async fn list(&self) -> Result<Vec<ModelCard>, Error> {
        let wire: ModelsWire = self
            .client
            .request(Method::GET, "/v1/models", None::<&()>)
            .await?;
        Ok(wire.models)
    }
}
