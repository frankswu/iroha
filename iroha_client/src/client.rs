use iroha::{crypto::KeyPair, prelude::*, torii::uri};
use iroha_derive::log;
use iroha_network::{prelude::*, Network};
use std::{
    convert::TryFrom,
    fmt::{self, Debug, Formatter},
};

pub struct Client {
    torii_url: String,
    key_pair: KeyPair,
}

/// Representation of `Iroha` client.
impl Client {
    pub fn new(configuration: &Configuration) -> Self {
        Client {
            torii_url: configuration.torii_url.clone(),
            key_pair: KeyPair::generate().expect("Failed to generate KeyPair."),
        }
    }

    /// Instructions API entry point. Submits one Iroha Special Instruction to `Iroha` peers.
    #[log]
    pub async fn submit(&mut self, instruction: Instruction) -> Result<(), String> {
        let network = Network::new(&self.torii_url);
        let transaction: RequestedTransaction =
            RequestedTransaction::new(vec![instruction], iroha::account::Id::new("root", "global"))
                .accept()?
                .sign(&self.key_pair)?
                .into();
        if let Response::InternalError = network
            .send_request(Request::new(
                uri::INSTRUCTIONS_URI.to_string(),
                Vec::from(&transaction),
            ))
            .await
            .map_err(|e| {
                format!(
                    "Error: {}, Failed to write a transaction request: {:?}",
                    e, &transaction
                )
            })?
        {
            return Err("Server error.".to_string());
        }
        Ok(())
    }

    /// Instructions API entry point. Submits several Iroha Special Instructions to `Iroha` peers.
    pub async fn submit_all(&mut self, instructions: Vec<Instruction>) -> Result<(), String> {
        let network = Network::new(&self.torii_url);
        let transaction: RequestedTransaction =
            RequestedTransaction::new(instructions, iroha::account::Id::new("root", "global"))
                .accept()?
                .sign(&self.key_pair)?
                .into();
        if let Response::InternalError = network
            .send_request(Request::new(
                uri::INSTRUCTIONS_URI.to_string(),
                Vec::from(&transaction),
            ))
            .await
            .map_err(|e| {
                format!(
                    "Error: {}, Failed to write a transaction request: {:?}",
                    e, &transaction
                )
            })?
        {
            return Err("Server error.".to_string());
        }
        Ok(())
    }

    /// Query API entry point. Requests queries from `Iroha` peers.
    #[log]
    pub async fn request(&mut self, request: &QueryRequest) -> Result<QueryResult, String> {
        let network = Network::new(&self.torii_url);
        match network
            .send_request(Request::new(uri::QUERY_URI.to_string(), request.into()))
            .await
            .map_err(|e| format!("Failed to write a get request: {}", e))?
        {
            Response::Ok(payload) => Ok(
                QueryResult::try_from(payload).expect("Failed to try Query Result from vector.")
            ),
            Response::InternalError => Err("Server error.".to_string()),
        }
    }
}

impl Debug for Client {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("public_key", &self.key_pair.public_key)
            .field("torii_url", &self.torii_url)
            .finish()
    }
}

pub mod maintenance {
    use super::*;
    use iroha::maintenance::*;

    impl Client {
        pub fn with_maintenance(configuration: &Configuration) -> MaintenanceClient {
            MaintenanceClient {
                client: Client::new(configuration),
            }
        }
    }

    #[derive(Debug)]
    pub struct MaintenanceClient {
        client: Client,
    }

    impl MaintenanceClient {
        #[log]
        pub async fn submit(&mut self, instruction: Instruction) -> Result<(), String> {
            self.client.submit(instruction).await
        }

        #[log]
        pub async fn submit_all(&mut self, instructions: Vec<Instruction>) -> Result<(), String> {
            self.client.submit_all(instructions).await
        }

        #[log]
        pub async fn request(&mut self, request: &QueryRequest) -> Result<QueryResult, String> {
            self.client.request(request).await
        }

        #[log]
        pub async fn health(&mut self) -> Result<Health, String> {
            let network = Network::new(&self.client.torii_url);
            match network
                .send_request(Request::new(uri::HEALTH_URI.to_string(), vec![]))
                .await
                .map_err(|e| format!("Failed to write a get request: {}", e))?
            {
                Response::Ok(payload) => {
                    Ok(Health::try_from(payload).expect("Failed to convert Health from vector."))
                }
                Response::InternalError => Err("Server error.".to_string()),
            }
        }
    }
}

pub mod assets {
    use super::*;
    use iroha::asset::query::GetAccountAssets;

    pub fn by_account_id(account_id: <Account as Identifiable>::Id) -> QueryRequest {
        GetAccountAssets::build_request(account_id)
    }
}
