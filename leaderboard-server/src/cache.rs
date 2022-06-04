use futures_util::future::BoxFuture;
use tracing::{debug, error};

#[derive(Clone)]
pub struct CacheService {
    inner: tower_redis::RedisService,
}

impl CacheService {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let url = std::env::var("CACHE_URL")?;
        let client = redis::Client::open(url)?;
        let connection = redis::aio::ConnectionManager::new(client).await?;
        let inner = tower_redis::RedisService::new(connection);

        Ok(Self { inner })
    }

    pub async fn get_cached<F>(&self, key: &str, op: F) -> Result<String, anyhow::Error>
    where
        F: FnOnce() -> BoxFuture<'static, Result<String, anyhow::Error>>,
    {
        match self.get(key).await {
            Ok(value) => Ok(value),
            Err(error) => {
                debug!("cache miss on key '{key}': {error:?}");

                let value = op().await?;

                if let Err(error) = self.set(key, value.as_str()).await {
                    error!("cache error on key '{key}': {error:?}");
                }

                Ok(value)
            }
        }
    }

    pub async fn get(&self, key: &str) -> Result<String, anyhow::Error> {
        let response = redis::from_redis_value(&self.inner.get(key).await?)?;

        Ok(response)
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<bool, anyhow::Error> {
        const SECONDS: usize = 600;
        let response = redis::from_redis_value(&self.inner.set_ex(key, value, SECONDS).await?)?;

        Ok(response)
    }
}
