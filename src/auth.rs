use std::rand;
use std::rand::Rng;
use iron::prelude::*;
use iron::response::modifiers::{Body, Status};
use iron::status;
use router::Router;
use urlencoded::UrlEncodedQuery;

use db;

fn authenticate(user: &str, pass: &str) -> Option<String> {
    if let Ok(realpass) = db::query_user(user, "pass") {
        if realpass == pass {
            let mut rng = rand::task_rng();
            let auth = format!("{:x}", rng.gen::<u64>());
            if let Ok(()) = db::set_user(user, "auth", &*auth) {
                if let Ok(()) = db::set_auth(&*auth, user) {
                    return Some(auth);
                }
            }
        }
    }
    None
}

fn auth_handler(req: &mut Request) -> IronResult<Response> {
    // Extract the decoded data as hashmap, using the UrlEncodedQuery plugin.
    if let Some(queries) = req.get_ref::<UrlEncodedQuery>() {
        if let (Some(user), Some(pass)) = (queries.get("user"), queries.get("pass")) {
            if let Some(auth) = authenticate(&*user[0], &*pass[0]) {
                return Ok(Response::new().set(Status(status::Ok)).set(Body(auth)));
            }
        }
    };
    Ok(Response::new().set(Status(status::Unauthorized)))
}

pub fn register_handler(router: &mut Router) {
    router.post("/api/auth", auth_handler);
}
