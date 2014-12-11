#![feature(globs)]

extern crate serialize;
extern crate redis;
extern crate iron;
extern crate router;
extern crate urlencoded;
extern crate bodyparser;

use router::Router;
use iron::Iron;

mod auth;
mod db;
mod mark;

fn main() {
    let mut router = Router::new();
    auth::register_handler(&mut router);
    mark::register_handler(&mut router);
    Iron::new(router).listen("localhost:3000").unwrap();
}
