use serialize::json;
use iron::prelude::*;
use iron::response::modifiers::{Body, Status};
use iron::status;
use router::Router;
use bodyparser::BodyParser;
use redis::Commands;
use db::db;

#[deriving(Decodable, Clone)]
struct MarkParams {
    student: uint,
    course: String,
    lab: uint,
    mark: Vec<uint>
}

// returns Some(Response) if data are NOT valid
// in this case Response is the error response with appropriate HTTP status
fn deny_if_not_valid_input(params: &MarkParams) -> Option<Response> {
    let body : Option<String> =
        if let Ok(1u) = db().sismember(format!("students"), params.student) {
            if let Ok(1u) = db().sismember(format!("student:{}:courses", params.student), &*params.course) {
                if let Ok(1u) = db().sismember(format!("course:{}:labs", params.course), params.lab) {
                    None
                } else {
                    Some(format!("Lab {} is not a valid lab in course {}", params.lab, params.course))
                }
            } else {
                Some(format!("Student {} is not enrolled in course {}", params.student, params.course))
            }
        } else {
            Some(format!("{} is not a valid student ID", params.student))
        };

    if let Some(reply) = body {
        Some(Response::new().set(Status(status::Forbidden)).set(Body(reply)))
    } else {
        None
    }
}

// FIXME: add protocol for repeated marking
// Currently, all marks are recorded in a list
fn mark_handler(req: &mut Request) -> IronResult<Response> {
    let parsed = req.get::<BodyParser<MarkParams>>();
    if let Some(params) = parsed {
        if let Some(response) = deny_if_not_valid_input(&params) {
            return Ok(response);
        }
        let key = format!("student:{}:{}:lab{}:marks", params.student, params.course, params.lab);
        if let Ok(()) = db().lpush(&*key, json::encode(&params.mark)) {
            return Ok(Response::new().set(Status(status::Ok))
                      .set(Body(format!("Mark {} has been entered for student {} course {} lab {}",
                                        params.mark, params.student, params.course, params.lab))));
        } else {
            return Ok(Response::new().set(Status(status::InternalServerError))
                      .set(Body(format!("Unable to write to database key {}", &*key))));
        }
    } else {
        return Ok(Response::new().set(Status(status::BadRequest))
                  .set(Body("Input is not valid JSON serialisation of MarkParams")));
    }
}

pub fn register_handler(router: &mut Router) {
    router.post("/api/mark", mark_handler);
}
