use std::error::Error;
use serialize::json;
use iron::prelude::*;
use iron::response::modifiers::{Body, Status};
use iron::status;
use router::{Params, Router};
use bodyparser::BodyParser;
use urlencoded::UrlEncodedQuery;
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

#[deriving(Encodable, Clone, Show)]
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

fn del_admin_info(name: &str) -> RedisResult<()> {
    let admin_info = get_admin_info(name);
    stop_if_error!(db().srem("admins", name));
    if let Some(ref courses) = admin_info.courses {
        stop_if_error!(db().del(format!("user:{}:courses", name)));
        for course in courses.iter() {
            stop_if_error!(db().hset(format!("course:{}", course), "admin", name));
        }
    }
    Ok(())
}

fn get_admin_handler(_: &mut Request) -> IronResult<Response> {
    match db().smembers::<&str, Vec<String>>("admins") {
        Ok(admins) => {
            let admin_info_vec : Vec<AdminParams> = admins.iter()
                .map(|admin| get_admin_info(admin.as_slice())).collect();
            Ok(Response::new().set(Status(status::Ok)).set(Body(json::encode(&admin_info_vec))))
        }
        Err(err) => {
                Ok(Response::new().set(Status(status::Forbidden))
                   .set(Body(format!("Failed to access db entry 'admins'. Reason {}.",
                                     err.description()))))
        }
    }
}

fn new_admin_handler(req: &mut Request) -> IronResult<Response> {
    if let Some(queries) = req.get_ref::<UrlEncodedQuery>() {
        if let Some(name) = queries.get("name") {
            return match db().sismember("admins", &*name[0]) {
                Ok(true) => Ok(Response::new().set(Status(status::Forbidden))
                               .set(Body(format!("Admin {} already exists.", name[0])))),
                Ok(false) => if let Ok(true) = db().sadd("admins", &*name[0]) {
                    Ok(Response::new().set(Status(status::Ok))
                       .set(Body(format!("User {} added as admin.", name[0]))))
                } else {
                    Ok(Response::new().set(Status(status::Forbidden))
                       .set(Body(format!("Failed to add user {} as admin.", name[0]))))
                },
                Err(err) => Ok(Response::new().set(Status(status::InternalServerError))
                               .set(Body(format!("Database access error: {}", err.description()))))
            }
        }
    }
    Ok(Response::new().set(Status(status::BadRequest))
           .set(Body("Invalid query string for /api/admin/new")))
}

fn del_admin_handler(req: &mut Request) -> IronResult<Response> {
    if let Some(queries) = req.get_ref::<UrlEncodedQuery>() {
        if let Some(user) = queries.get("name") {
            return match del_admin_info(&*user[0]) {
                Ok(()) => Ok(Response::new().set(Status(status::Ok)).set(Body(
                    format!("Removed user {} from admins", user[0])))),
                Err(err) => Ok(Response::new().set(Status(status::Forbidden)).set(Body(
                    format!("Failed to remove user {} from admins: {}",
                            user[0], err.description()))))
            }
        }
    }
    Ok(Response::new().set(Status(status::BadRequest))
       .set(Body("Invalid query string for /api/admin/del")))
}

fn new_course_handler(req: &mut Request) -> IronResult<Response> {
    Ok(Response::new())
}

fn add_marker_handler(req: &mut Request) -> IronResult<Response> {
     let ref course = req.extensions.get::<Router, Params>().unwrap().find("course").unwrap_or("/");

}

pub fn register_handler(router: &mut Router) {
    router.post("/api/admin/get", get_admin_handler);
    router.post("/api/admin/new", new_admin_handler);
    router.post("/api/admin/del", del_admin_handler);
    router.post("/api/course/new", new_course_handler);
    router.post("/api/course/:course/add-marker", add_marker_handler);
}
