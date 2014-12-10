extern crate redis;

use self::redis::{Commands, RedisResult};

fn con() -> redis::Connection {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    client.get_connection().unwrap()
}

pub fn query_user(name: &str, field: &str) -> RedisResult<String> {
    con().hget(format!("user:{}", name), field)
}

pub fn set_user(name: &str, field: &str, value: &str) -> RedisResult<()> {
    con().hset(format!("user:{}", name), field, value)
}
