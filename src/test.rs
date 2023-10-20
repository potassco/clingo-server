use super::rocket;

use rocket::http::ContentType;
use rocket::http::Status;
use rocket::local::blocking::Client;
use serde_json::Value;

#[test]
fn test_create() {
    let client = Client::tracked(rocket()).unwrap();
    let response = client.get("/create").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_string(),
        Some("Created clingo Solver.".into())
    );
    let response = client.get("/register_dl_theory").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_string(),
        Some("Difference logic theory registered.".into())
    );
    let response = client.post("/add").body("a.").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string(), Some("Added data to Solver.".into()));
    let response = client
        .post("/ground")
        .header(ContentType::JSON)
        .body("{\"base\":[]}")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string(), Some("Grounding.".into()));
    let response = client.get("/solve").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string(), Some("Solving.".into()));
    let mut response = client.get("/model").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let mut body_string = response.into_string();
    while body_string == Some("\"Running\"".into()) {
        response = client.get("/model").dispatch();
        body_string = response.into_string();
    }
    // assert_eq!(response.status(), Status::Ok);
    let data = body_string.unwrap();
    let data: Value = serde_json::from_str(&data).unwrap();
    assert_eq!(
        data["Model"],
        Value::Array(vec![Value::Number(97.into()), Value::Number(10.into())])
    );

    let response = client.get("/resume").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string(), Some("Search is resumed.".into()));
    let response = client.get("/close").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string(), Some("Solve handle closed.".into()));
    let response = client.get("/statistics").dispatch();
    assert_eq!(response.status(), Status::Ok);
}
#[test]
fn test_register_dl_theory() {
    let client = Client::tracked(rocket()).unwrap();
    let response = client.get("/register_dl_theory").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.into_string().unwrap();
    assert_eq!(
        &data,
        "{\"InternalError\":\"Solver::register_dl_theory failed! No control object.\"}"
    );
}
#[test]
fn test_add() {
    let client = Client::tracked(rocket()).unwrap();
    let response = client.post("/add").body("body.").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.into_string().unwrap();
    assert_eq!(
        &data,
        "{\"InternalError\":\"Solver::add failed! No control object.\"}"
    );
}
#[test]
fn test_ground() {
    let client = Client::tracked(rocket()).unwrap();
    let response = client
        .post("/ground")
        .header(ContentType::JSON)
        .body("{\"base\":[]}")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.into_string().unwrap();
    assert_eq!(
        &data,
        "{\"InternalError\":\"Solver::ground failed! No control object.\"}"
    );
}
#[test]
fn test_solve() {
    let client = Client::tracked(rocket()).unwrap();
    let response = client.get("/solve").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.into_string().unwrap();
    assert_eq!(
        &data,
        "{\"InternalError\":\"Solver::solve failed! No control object.\"}"
    );
}
#[test]
fn test_model() {
    let client = Client::tracked(rocket()).unwrap();
    let response = client.get("/model").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.into_string().unwrap();
    assert_eq!(
        &data,
        "{\"InternalError\":\"Solver::model failed! No SolveHandle.\"}"
    );
}
#[test]
fn test_resume() {
    let client = Client::tracked(rocket()).unwrap();
    let response = client.get("/resume").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.into_string().unwrap();
    assert_eq!(
        &data,
        "{\"InternalError\":\"Solver::resume failed! No SolveHandle.\"}"
    );
}
#[test]
fn test_close() {
    let client = Client::tracked(rocket()).unwrap();
    let response = client.get("/close").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.into_string().unwrap();
    assert_eq!(
        &data,
        "{\"InternalError\":\"Solver::close failed! Solver is not running.\"}"
    );
}
#[test]
fn test_statistics() {
    let client = Client::tracked(rocket()).unwrap();
    let response = client.get("/statistics").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.into_string().unwrap();
    assert_eq!(
        &data,
        "{\"InternalError\":\"Solver::statistics failed! No control object.\"}"
    );
}
