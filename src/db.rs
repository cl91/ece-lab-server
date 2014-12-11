use redis::{Client, Connection, Commands, RedisResult};

pub fn db() -> Connection {
    let client = Client::open("redis://127.0.0.1/").unwrap();
    client.get_connection().unwrap()
}

pub fn query_user(name: &str, field: &str) -> RedisResult<String> {
    db().hget(format!("user:{}", name), field)
}

pub fn set_user(name: &str, field: &str, value: &str) -> RedisResult<()> {
    db().hset(format!("user:{}", name), field, value)
}

pub fn set_auth(auth: &str, user: &str) -> RedisResult<()> {
    db().hset("auth", auth, user)
}
