use crate::responses::error_msgs::Error;
use bitcoincore_rpc::{Auth, Client as RpcClient, Error as RpcError, RpcApi, jsonrpc::Error as JsonRpcError, jsonrpc::serde_json::Value};
use crypsol_logger::log;
use log::Level;

pub async fn make_rpc_call<T: for<'de> serde::Deserialize<'de>>(rpc_url: &str, rpc_user: &str, rpc_pass: &str, method: &str, params_value: Vec<Value>) -> Result<T, Error> {
    let rpc = match RpcClient::new(rpc_url, Auth::UserPass(rpc_user.to_string(), rpc_pass.to_string())) {
        Ok(rpc) => rpc,
        Err(e) => {
            log!(Level::Error, "Failed to create RPC client: {:?}", e);
            return Err(Error::RpcIssue);
        }
    };

    match rpc.call::<Value>(method, &params_value) {
        Ok(value) => match serde_json::from_value(value) {
            Ok(value) => Ok(value),
            Err(e) => {
                log!(Level::Error, "Failed to deserialize RPC response: {:?}", e);
                Err(Error::RpcIssue)
            }
        },
        Err(RpcError::JsonRpc(JsonRpcError::Rpc(e))) => {
            if e.code == -3 {
                log!(Level::Error, "Invalid amount RPC call Error: {:?}", e);
                Err(Error::InvalidAmount)
            } else if e.code == -5 {
                log!(Level::Error, "Invalid amount RPC call Error: {:?}", e);
                Err(Error::InvalidAddress)
            } else {
                log!(Level::Error, "RPC call failed: {:?}", e);
                Err(Error::RpcIssue)
            }
        }
        Err(e) => {
            log!(Level::Error, "RPC call failed: {:?}", e);
            Err(Error::RpcIssue)
        }
    }
}
