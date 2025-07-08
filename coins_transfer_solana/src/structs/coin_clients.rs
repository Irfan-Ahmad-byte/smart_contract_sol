use crate::responses::error_msgs::Error;
use crate::services::configs::get_a_config;
use crate::services::rpc_client::RpcClient;
use deadpool_redis::Pool;
use sqlx::PgPool;

pub struct LitecoinClient;

impl LitecoinClient {
    pub async fn new(pool: &PgPool, redis_pool: &Pool) -> Result<RpcClient, Error> {
        dotenvy::dotenv().ok();

        let rpc_url = get_a_config(pool, redis_pool, "LITECOIN_RPC_URL".to_string()).await?;

        let rpc_user = get_a_config(pool, redis_pool, "LITECOIN_RPC_USER".to_string()).await?;
        let rpc_pass = get_a_config(pool, redis_pool, "LITECOIN_RPC_PASSWORD".to_string()).await?;

        Ok(RpcClient { rpc_url, rpc_user, rpc_pass })
    }
}