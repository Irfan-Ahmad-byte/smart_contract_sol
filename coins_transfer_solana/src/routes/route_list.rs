use actix_web::web;

use crate::handlers::configs::{
    create_config_handler, delete_all_config_handler, delete_config_handler, get_config_handler, list_configs_handler, update_config_handler,
};
use crate::handlers::conversion_rates::conversion_rate_get_handler;
use crate::handlers::deposits::{
    get_transaction_history_handler,
};
use crate::handlers::health::health_check;
use crate::handlers::notify_webhooks::validate_usdt_payment;
use crate::handlers::users::{create_user_handler, rollback_user_creation_handler};
use crate::handlers::coins::{create_coin_handler, get_all_coins_handler, address_generation_handler};
use crate::handlers::wallets::{get_wallet_handler, update_wallet_handler};
use crate::handlers::withdrawals::{
    create_withdrawal_handler, rollback_withdrawal_request_handler, get_withdrawal_history_handler,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(
                web::scope("/health")
                    .route("/", web::post().to(health_check))
                    .route("/", web::get().to(health_check)),
            )
            .service(
                web::scope("/configs")
                    .route("/", web::post().to(create_config_handler))
                    .route("/", web::get().to(list_configs_handler))
                    .route("/{name}", web::get().to(get_config_handler))
                    .route("/{name}", web::put().to(update_config_handler))
                    .route("/{name}", web::delete().to(delete_config_handler))
                    .route("/", web::delete().to(delete_all_config_handler)),
            )
            .service(
                web::scope("/users")
                    .route("/", web::post().to(create_user_handler))
                    .route("/rollback/{event_id}", web::post().to(rollback_user_creation_handler)),
            )
            .service(
                web::scope("/coins")
                    .route("/", web::post().to(create_coin_handler))
                    .route("/", web::get().to(get_all_coins_handler))
                    .route("/rate/{symbol}", web::get().to(conversion_rate_get_handler))
                    .route("/address/new", web::get().to(address_generation_handler)),
            )
            .service(
                web::scope("/wallets")
                    .route("/", web::get().to(get_wallet_handler))
                    .route("/{wallet_id}", web::put().to(update_wallet_handler)),
            )
            .service(
                web::scope("/withdrawals")
                    .route("/", web::post().to(create_withdrawal_handler))
                    .route("/rollback/{event_id}", web::post().to(rollback_withdrawal_request_handler))
                    .route("/history", web::get().to(get_withdrawal_history_handler)),
            )
            .service(
                web::scope("/deposits")
                    .route("/usdt/validate_deposit", web::post().to(validate_usdt_payment))
                    .route("/history", web::get().to(get_transaction_history_handler)),
            ),
    )
        .route("/", web::get().to(health_check));
}
