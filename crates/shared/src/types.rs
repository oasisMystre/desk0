#[cfg(feature = "redis")]
pub type RedisPool = deadpool_redis::Pool;
#[cfg(feature = "postgres")]
pub type PgPool = diesel_async::pooled_connection::bb8::Pool<diesel_async::AsyncPgConnection>;
