use std::{any::Any, fmt::Debug};

use super::rocket;

use rocket::http::Status;
use rocket::local::Client;

#[test]
fn test_create() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/create").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some("Created clingo Solver.".into())
    );
    let mut response = client.get("/register_dl_theory").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some("Difference logic theory registered.".into())
    );
    let mut response = client.post("/add").body("body.").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some("Added data to Solver.".into())
    );
}
#[test]
fn test_register_dl_theory() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/register_dl_theory").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some("InternalError: Solver::register_dl_theory failed! No Control object.".into())
    );
}
#[test]
fn test_add() {
    let client = Client::new(rocket()).unwrap();
    let request = client.post("/add").body("body.");
    let mut response = request.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some("InternalError: Solver::add failed! No control object.".into())
    );
}