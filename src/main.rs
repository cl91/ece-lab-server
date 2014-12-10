#![feature(globs)]

extern crate iron;
extern crate router;
extern crate urlencoded;

use router::Router;
use iron::Iron;

mod auth;
mod db;

fn main() {
    let mut router = Router::new();
    auth::register_handler(&mut router);
    Iron::new(router).listen("localhost:3000");
}
