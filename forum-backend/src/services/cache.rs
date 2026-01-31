use redis::AsyncCommands;

pub struct CacheService {
    client: redis::Client,
}

impl CacheService {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection, redis::RedisError> {
        self.client.get_multiplexed_async_connection().await
    }

    pub async fn check_rate_limit(
        &self,
        key: &str,
        max_requests: u64,
        window_secs: u64,
    ) -> Result<bool, redis::RedisError> {
        let mut conn = self.get_connection().await?;

        let current: u64 = conn.incr(key, 1u64).await?;

        if current == 1 {
            let _: () = conn.expire(key, window_secs as i64).await?;
        }

        Ok(current <= max_requests)
    }

    pub async fn cache_get(&self, key: &str) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.get_connection().await?;
        conn.get(key).await
    }

    pub async fn cache_set(
        &self,
        key: &str,
        value: &str,
        ttl_secs: u64,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.get_connection().await?;
        conn.set_ex(key, value, ttl_secs).await
    }
}
