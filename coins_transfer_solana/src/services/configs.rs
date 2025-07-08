use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;
use sqlx::{PgPool, query, query_as};

use crate::cache::configs::{delete_config_cache, drop_a_config_from_all_config_cache, get_all_configs_from_cache, get_config_by_name_from_cache, increment_all_config_cache, set_all_config_cache, set_config_cache};
use crate::entities::configs::Configs;
use crate::responses::error_msgs::Error;
use crate::responses::success_msgs::SuccessMessages;
use crate::structs::configs::{ConfigCreate, ConfigUpdate};
use crate::utils::time::TimeHandler;

pub async fn create_config(pool: &PgPool, redis_pool: &Pool, configs: ConfigCreate) -> Result<SuccessMessages, Error> {
    let time_handler = TimeHandler::new();
    let now = time_handler.get_current_time().naive_utc();

    let result = query!(
        r#"
        INSERT INTO configs (name, value, created_at, updated_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (name) DO NOTHING
        RETURNING id, created_at
        "#,
        configs.name.clone(),
        configs.value.clone(),
        now,
        now
    )
    .fetch_one(pool)
    .await;

    let new_config = match result {
        Ok(record) => Configs { id: record.id, name: configs.name.clone(), value: configs.value, created_at: record.created_at, updated_at: record.created_at },
        Err(sqlx::Error::RowNotFound) => {
            log!(Level::Error, "Duplicate Entry: Config with ID {} already exists", configs.name);
            return Err(Error::DuplicateEntry);
        }
        Err(e) => {
            log!(Level::Error, "Database error on insert: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };

    let _ = set_config_cache(redis_pool, &new_config).await;
    let _ = increment_all_config_cache(redis_pool, &new_config).await;

    Ok(SuccessMessages::CreatedConfig { config_id: new_config.id })
}

pub async fn get_config_by_name(pool: &PgPool, redis_pool: &Pool, name: String) -> Result<SuccessMessages, Error> {
    if let Some(configs) = get_config_by_name_from_cache(redis_pool, &name).await {
        return Ok(SuccessMessages::FoundConfig {
            config_id: 0, // use 0 to indicate a list, as there is no specific ID
            configs_list: Some(vec![configs]),
        });
    }

    let configs = get_from_env(&name).await;
    if configs.is_some() {
        let config = configs.unwrap();
        if !config.is_empty() {
            let configs = Configs { id: 0, name, value: config, created_at: TimeHandler::new().get_current_time().naive_utc(), updated_at: TimeHandler::new().get_current_time().naive_utc() };
            return Ok(SuccessMessages::FoundConfig {
                config_id: 0, // use 0 to indicate a list, as there is no specific ID
                configs_list: Some(vec![configs]),
            });
        }
    }

    let configs = get_a_config_from_db(pool, &name).await?;

    let _ = set_config_cache(redis_pool, &configs).await;
    let _ = increment_all_config_cache(redis_pool, &configs).await;

    Ok(SuccessMessages::FoundConfig { config_id: configs.id, configs_list: Some(vec![configs]) })
}

#[allow(dead_code, unused)]
pub async fn get_a_config(pool: &PgPool, redis_pool: &Pool, name: String) -> Result<String, Error> {
    let conf = match get_config_by_name(pool, redis_pool, name.clone()).await {
        Ok(SuccessMessages::FoundConfig { config_id, configs_list }) => {
            let _ = config_id;
            if let Some(config) = configs_list {
                config[0].value.clone()
            } else {
                return Err(Error::EnvVarMissing(name));
            }
        }
        Ok(_) => return Err(Error::EnvVarMissing(name)),
        Err(_) => {
            log!(Level::Error, "{} is not set in .env file", name);
            return Err(Error::EnvVarMissing(name));
        }
    };

    Ok(conf)
}

pub async fn get_a_config_from_db(pool: &PgPool, name: &String) -> Result<Configs, Error> {
    let result = query_as!(
        Configs,
        r#"
        SELECT id, name, value, created_at, updated_at FROM configs WHERE name = $1
        "#,
        name.clone()
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(configs) => Ok(configs),
        Err(sqlx::Error::RowNotFound) => {
            log!(Level::Error, "Config with name {} not found in db", name);
            Err(Error::NotFound(format!("Config with name {name} not found in db")))
        }
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn get_all_configs(pool: &PgPool, redis_pool: &Pool) -> Result<SuccessMessages, Error> {
    if let Some(configs) = get_all_configs_from_cache(redis_pool).await {
        return Ok(SuccessMessages::FoundConfig {
            config_id: 0, // use 0 to indicate a list, as there is no specific ID
            configs_list: Some(configs),
        });
    }

    let result = query_as!(
        Configs,
        r#"
        SELECT id, name, value, created_at, updated_at FROM configs
        "#
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(configs) => {
            let _ = set_all_config_cache(redis_pool, &configs).await;
            Ok(SuccessMessages::FoundConfig { config_id: 0, configs_list: Some(configs) })
        }
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn update_config(pool: &PgPool, redis_pool: &Pool, name: String, configs_update: ConfigUpdate) -> Result<SuccessMessages, Error> {
    let configs = get_a_config_from_db(pool, &name).await?;

    let time_handler = TimeHandler::new();
    let now = time_handler.get_current_time().naive_utc();

    let update_result = query!(
        r#"
        UPDATE configs
        SET name = COALESCE($1, name),
            value = COALESCE($2, value),
            updated_at = $3
        WHERE name = $4
        RETURNING id, updated_at
        "#,
        configs_update.name.clone(),
        configs_update.value.clone(),
        now,
        name.clone()
    )
    .fetch_one(pool)
    .await;

    match update_result {
        Ok(record) => {
            let updated_configs = Configs { id: record.id, name, created_at: configs.created_at, updated_at: record.updated_at, value: configs_update.value.unwrap_or(configs.value.clone()) };
            let _ = drop_a_config_from_all_config_cache(redis_pool, &configs).await;
            let _ = set_config_cache(redis_pool, &updated_configs).await;
            let _ = increment_all_config_cache(redis_pool, &updated_configs).await;

            Ok(SuccessMessages::UpdatedConfig { config_id: configs.id, configs_list: Some(vec![updated_configs]) })
        }
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn delete_config(pool: &PgPool, redis_pool: &Pool, name: String) -> Result<SuccessMessages, Error> {
    let delete_result = query_as!(
        Configs,
        r#"
        DELETE FROM configs WHERE name = $1
        RETURNING id, name, value, created_at, updated_at
        "#,
        name.clone()
    )
    .fetch_one(pool)
    .await;

    match delete_result {
        Ok(config) => {
            let _ = delete_config_cache(redis_pool, &name).await;
            let _ = drop_a_config_from_all_config_cache(redis_pool, &config).await;
            Ok(SuccessMessages::DeletedConfig { config: name })
        }
        Err(sqlx::Error::RowNotFound) => {
            log!(Level::Error, "Config with name {} not found", name);
            Err(Error::NotFound(format!("Config with name {name} not found")))
        }
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn delete_all_configs(pool: &PgPool, redis_pool: &Pool) -> Result<SuccessMessages, Error> {
    let delete_result = query_as!(
        Configs,
        r#"
        DELETE FROM configs
        RETURNING id, name, value, created_at, updated_at
        "#
    )
    .fetch_all(pool)
    .await;

    match delete_result {
        Ok(configs) => {
            for config in configs {
                let _ = delete_config_cache(redis_pool, &config.name).await;
                let _ = drop_a_config_from_all_config_cache(redis_pool, &config).await;
            }
            Ok(SuccessMessages::DeletedAllConfigs)
        }
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn get_from_env(config_name: &str) -> Option<String> {
    let val = std::env::var(config_name);

    if val.is_err() {
        return None;
    }
    let val = val.unwrap();
    Some(val)
}

#[allow(dead_code, unused)]
pub async fn parse_int(config_val: String) -> Result<i32, Error> {
    let val = config_val.parse::<i32>();
    match val {
        Ok(v) => Ok(v),
        Err(e) => {
            log!(Level::Error, "error parsing int from config value: {}", e);
            Err(Error::InvalidConfiguration)
        }
    }
}

#[allow(dead_code, unused)]
pub async fn parse_float(config_val: String) -> Result<f32, Error> {
    let val = config_val.parse::<f32>();
    match val {
        Ok(v) => Ok(v),
        Err(e) => {
            log!(Level::Error, "error parsing float from config value: {}", e);
            Err(Error::InvalidConfiguration)
        }
    }
}
