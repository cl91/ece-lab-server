use std::error::Error;
use serialize::json;
use iron::prelude::*;
use iron::response::modifiers::{Body, Status};
use iron::status;
use router::Router;
use bodyparser::BodyParser;
use redis::{Commands, RedisResult};
use db::db;

macro_rules! stop_if_error {
    ($rhs: expr) => {
        {
            let res : RedisResult<uint> = $rhs;
            match res {
                Err(err) => { return Err(err); }
                _ => {}
            }
        }
    }
}

#[deriving(Encodable, Decodable, Clone, Show)]
struct AdminParams {
    name: String,
    courses: Option<Vec<String>>
}

fn get_admin_info(admin: &str) -> AdminParams {
    if let Ok(courses) = db().smembers::<String, Vec<String>>(format!("user:{}:courses", admin)) {
        AdminParams { name: admin.to_string(), courses: Some(courses) }
    } else {
        AdminParams { name: admin.to_string(), courses: None }
    }
}

fn set_admin_info(admin: &AdminParams) -> RedisResult<()> {
    stop_if_error!(db().sadd("admins", &*admin.name));
    stop_if_error!(db().del(format!("user:{}:courses", admin.name)));
    for course in admin.courses.iter() {
        stop_if_error!(db().sadd(format!("user:{}:courses", admin.name), course.as_slice()));
    }
    Ok(())
}

fn get_handler(_: &mut Request) -> IronResult<Response> {
    match db().smembers::<&str, Vec<String>>("admins") {
        Ok(admins) => {
            let admin_info_vec : Vec<AdminParams> = admins.iter()
                .map(|admin| get_admin_info(admin.as_slice())).collect();
            Ok(Response::new().set(Status(status::Ok)).set(Body(json::encode(&admin_info_vec))))
        }
        Err(err) => {
                Ok(Response::new().set(Status(status::Forbidden))
                   .set(Body(format!("Failed to access db entry 'admin'. Reason {}.",
                                     err.description()))))
        }
    }
}

fn set_handler(req: &mut Request) -> IronResult<Response> {
    if let Some(admin) = req.get::<BodyParser<AdminParams>>() {
        match set_admin_info(&admin) {
            Ok(_) => {
                Ok(Response::new().set(Status(status::Ok))
                   .set(Body(format!("Successfully updated database for admin {}", admin.name))))
            }
            Err(err) => {
                Ok(Response::new().set(Status(status::Forbidden))
                   .set(Body(format!("Failed to update database for admin {}. Reason {}",
                                     admin.name, err.description()))))
            }
        }
    } else {
        Ok(Response::new().set(Status(status::BadRequest))
           .set(Body("Invalid JSON input for /api/set/admin")))
    }
}

pub fn register_handler(router: &mut Router) {
    router.post("/api/get/admin", get_handler);
    router.post("/api/set/admin", set_handler);
}
