use anyhow::{Error, Result};
use redis::{aio::Connection, AsyncCommands, FromRedisValue};
use serde::de::DeserializeOwned;

pub extern crate redis;

/**
 * Get a redis connection
 */
pub async fn connect(url: &str) -> Result<Connection, Error> {
    let client = redis::Client::open(url);
    let client = match client {
        Ok(client) => client,
        Err(e) => return Err(Error::new(e)),
    };

    let con = client.get_tokio_connection().await;
    let con = match con {
        Ok(con) => con,
        Err(e) => return Err(Error::new(e)),
    };

    return Ok(con);
}

/**
 * Set a redis value
 */
pub async fn set(key: &str, value: &str, con: &mut Connection) -> Result<(), Error> {
    let _: () = con.set(key, value).await?;

    return Ok(());
}

/**
 * Set a redis value with a ttl
 */
pub async fn set_with_ttl(
    key: &str,
    value: &str,
    ttl: usize,
    con: &mut Connection,
) -> Result<(), Error> {
    let _: () = con.set_ex(key, value, ttl).await?;

    return Ok(());
}

/**
 * Get a redis value
 */
pub async fn get<T: FromRedisValue>(key: &str, con: &mut Connection) -> Result<Option<T>, Error> {
    let value: Option<T> = con.get(key).await?;

    return Ok(value);
}

/**
 * Get a redis value based on a pattern
 */
pub async fn mget<T: FromRedisValue>(pattern: &str, con: &mut Connection) -> Result<Vec<T>, Error> {
    let keys: Vec<String> = con.keys(pattern).await?;
    let values: Vec<T> = con.mget(keys).await?;

    return Ok(values);
}

/**
 * Get a redis value as deserialized type
 */
pub async fn get_as<T>(key: &str, con: &mut Connection) -> Result<Option<T>, Error>
where
    T: DeserializeOwned,
{
    let value: Option<String> = con.get(key).await?;

    return match value {
        Some(value) => {
            let value: T = serde_json::from_str(&value)?;
            Ok(Some(value))
        }
        None => Ok(None),
    };
}

/**
 * Get a redis value as deserialized type vec
 */
pub async fn mget_as<T>(pattern: &str, con: &mut Connection) -> Result<Vec<T>, Error>
where
    T: DeserializeOwned,
{
    let keys: Vec<String> = con.keys(pattern).await?;
    let values: Vec<String> = con.mget(keys).await?;
    let values: Vec<T> = values
        .iter()
        .map(|value| serde_json::from_str(value))
        .collect::<Result<Vec<T>, _>>()?;

    return Ok(values);
}

/**
 * Delete a redis value
 */
pub async fn delete(key: &str, con: &mut Connection) -> Result<(), Error> {
    let _: () = con.del(key).await?;

    return Ok(());
}
