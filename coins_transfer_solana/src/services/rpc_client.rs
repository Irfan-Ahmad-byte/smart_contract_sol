use crate::responses::error_msgs::Error;
use crate::services::configs::get_a_config;
use crate::utils::rpc::make_rpc_call;
use deadpool_redis::Pool;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use sqlx::PgPool;

#[derive(Clone)]
pub struct RpcClient {
    pub rpc_url: String,
    pub rpc_user: String,
    pub rpc_pass: String,
}

impl RpcClient {
    pub async fn new(pool: &PgPool, redis_pool: &Pool, coin: &str) -> Result<Self, Error> {
        let rpc_url = get_a_config(pool, redis_pool, format!("{coin}_RPC_URL")).await?;
        let rpc_user = get_a_config(pool, redis_pool, format!("{coin}_RPC_USER")).await?;
        let rpc_pass = get_a_config(pool, redis_pool, format!("{coin}_RPC_PASSWORD")).await?;

        Ok(RpcClient { rpc_url, rpc_user, rpc_pass })
    }

    pub async fn make_rpc_call<T: DeserializeOwned>(&self, method: &str, params: Vec<Value>) -> Result<T, Error> {
        make_rpc_call(&self.rpc_url, &self.rpc_user, &self.rpc_pass, method, params).await
    }

    pub async fn generate_new_address(&self, label: &str) -> Result<String, Error> {
        let params = vec![json!(label)];
        // let params_raw = serde_json::value::to_raw_value(&params).unwrap();
        self.make_rpc_call("getnewaddress", params).await
    }

    pub async fn validate_address(&self, address: &str) -> Result<bool, Error> {
        let params = vec![json!(address)];
        let result: serde_json::Value = self.make_rpc_call("validateaddress", params).await?;
        Ok(result["isvalid"].as_bool().unwrap_or(false))
    }

    pub async fn send_to_address(&self, address: &str, amount: f64) -> Result<String, Error> {
        // let amount = amount.to_string();
        let params = vec![json!(address), json!(amount)];
        self.make_rpc_call("sendtoaddress", params).await
    }

    pub async fn get_unconfirmed_balance(&self, address: &str) -> Result<(f64, i64), Error> {
        // Min and max confirmations for listunspent
        let min_conf = 0;
        let max_conf = 1;

        // Use listunspent RPC call to get UTXOs for the specific address
        let params = vec![json!(min_conf), json!(max_conf), json!(vec![json!(address)])];
        let utxos: Vec<serde_json::Value> = self.make_rpc_call("listunspent", params).await?;

        // Sum the amounts of the UTXOs to get the unconfirmed balance
        let unconfirmed_balance = utxos.iter().fold(0.0, |acc, utxo| acc + utxo["amount"].as_f64().unwrap_or(0.0));

        // Assuming that all UTXOs for the given address will have the same number of confirmations,
        // we take the confirmations from the first UTXO.
        let confirmations = utxos.first().map_or(0, |utxo| utxo["confirmations"].as_i64().unwrap_or(0));

        Ok((unconfirmed_balance, confirmations))
    }

    pub async fn get_transaction(&self, tx_id: &str) -> Result<(Vec<serde_json::Value>, i64), Error> {
        let method = "gettransaction";
        let params = vec![json!(tx_id)];
        let response: serde_json::Value = self.make_rpc_call(method, params).await?;

        let details = response["details"].as_array().unwrap().clone();
        let confirmations = response["confirmations"].as_i64().unwrap();

        Ok((details, confirmations))
    }

    pub async fn get_balance(&self) -> Result<f64, Error> {
        self.make_rpc_call("getbalance", vec![]).await
    }
}
