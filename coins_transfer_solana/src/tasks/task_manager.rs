use crate::tasks::rate_updater::start_rate_updater;
use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Once};
use tokio::task;
use tokio::task::JoinHandle;

// Extend the TaskManager struct to include dynamic task registration
pub struct TaskManager {
    pub redis_pool: Option<Pool>,
    pub pg_pool: Option<PgPool>,
    pub tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>, // To track running tasks
}

impl TaskManager {
    pub fn new(redis_pool: Option<Pool>, pg_pool: Option<PgPool>) -> Self {
        Self { redis_pool, pg_pool, tasks: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Start the task manager and register initial tasks
    pub fn start_manager(&self) {
        log!(Level::Info, "🌐 Start Task Manager 🌐");
        static START_ONCE: Once = Once::new();
        START_ONCE.call_once(|| {
            self.start_task("update_conversion_rate", |pg_pool, redis_pool| async move {
                Self::update_conversion_rate(pg_pool, redis_pool).await;
            });

            log!(Level::Info, "🌐 Task Manager Started ✅");
        });
    }

    /// Dynamically add a new task
    pub fn add_task<F, Fut>(&self, task_name: &str, task_function: F)
    where
        F: Fn(PgPool, Pool) -> Fut + Send + 'static + Sync,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let redis_pool = self.redis_pool.clone();
        let pg_pool = self.pg_pool.clone();

        match (redis_pool, pg_pool) {
            (Some(redis_pool), Some(pg_pool)) => {
                let task_name = task_name.to_string();
                let tasks_arc = self.tasks.clone();

                // Spawn a new task
                let handle = task::spawn({
                    let task = task_name.clone();
                    async move {
                        loop {
                            TaskManager::run_task(&task, &task_function, pg_pool.clone(), redis_pool.clone()).await;
                            tokio::time::sleep(std::time::Duration::from_secs(86400)).await;
                            // Daily interval
                        }
                    }
                });

                // Register the task in the task map
                let mut tasks = tasks_arc.lock().unwrap();
                tasks.insert(task_name.clone(), handle);
            }
            _ => {
                log!(Level::Error, "Redis pool or PgPool not initialized.");
            }
        }
    }

    fn start_task<F, Fut>(&self, task_name: &'static str, task_function: F)
    where
        F: Fn(PgPool, Pool) -> Fut + Send + 'static + Sync,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let redis_pool = self.redis_pool.clone();
        let pg_pool = self.pg_pool.clone();

        match (redis_pool, pg_pool) {
            (Some(redis_pool), Some(pg_pool)) => {
                task::spawn(async move {
                    loop {
                        TaskManager::run_task(task_name, &task_function, pg_pool.clone(), redis_pool.clone()).await;
                        tokio::time::sleep(std::time::Duration::from_secs(86400)).await;
                        // Daily interval
                    }
                });
            }
            _ => {
                log!(Level::Error, "Redis pool or PgPool not initialized.");
            }
        }
    }

    async fn run_task<F, Fut>(task_name: &str, task_function: &F, pg_pool: PgPool, redis_pool: Pool)
    where
        F: Fn(PgPool, Pool) -> Fut + Send + 'static + Sync,
        Fut: Future<Output = ()> + Send,
    {
        log!(Level::Info, "Starting JOB ................. {} ................", task_name);
        task_function(pg_pool, redis_pool.clone()).await;

        // match get_cache(&redis_pool, task_name).await {
        //     Ok(e) => {
        //         if e.is_some() {
        //             log!(Level::Info, "{} JOB is already running.", task_name);
        //         } else {
        //             log!(Level::Info, "No running {} JOB found, starting new job .............", task_name);
        //             match set_cache(&redis_pool, task_name, &true, Some(300)).await {
        //                 Ok(_) => {
        //                     task_function(pg_pool, redis_pool.clone()).await;
        //                     delete_cache(&redis_pool, task_name).await.unwrap();
        //                 }
        //                 Err(e) => {
        //                     log!(Level::Error, "Error setting {} key into cache: {:?}", task_name, e);
        //                 }
        //             }
        //         }
        //     }
        //     Err(e) => {
        //         log!(Level::Error, "Error getting {} key from cache: {:?}", task_name, e);
        //     }
        // }
    }

    async fn update_conversion_rate(pg_pool: PgPool, redis_pool: Pool) {
        start_rate_updater(pg_pool, redis_pool).await;
    }

    /// Stop a task dynamically
    #[allow(dead_code, unused)]
    pub fn stop_task(&self, task_name: &str) {
        let mut tasks = self.tasks.lock().unwrap();
        match tasks.remove(task_name) {
            Some(handle) => {
                handle.abort();
                log!(Level::Info, "Task {} has been stopped.", task_name);
            }
            _ => {
                log!(Level::Info, "No task with name {} is running.", task_name);
            }
        }
    }
}
