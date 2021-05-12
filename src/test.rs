use super::rocket;

use rocket::http::Status;
use rocket::local::Client;
use serde_json::Value;

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
    let mut response = client.post("/add").body("a.").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("Added data to Solver.".into()));
    let mut response = client.get("/ground").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("Grounding.".into()));
    let mut response = client.get("/solve").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("Solving.".into()));
    let mut response = client.get("/model").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let mut body_string = response.body_string();
    while body_string == Some("\"Running\"".into()) {
        response = client.get("/model").dispatch();
        body_string = response.body_string();
    }
    assert_eq!(response.status(), Status::Ok);
    let data = body_string.unwrap();
    let data: Value = serde_json::from_str(&data).unwrap();
    assert_eq!(
        data["Model"],
        Value::Array(vec![Value::Number(97.into()), Value::Number(10.into())])
    );

    let mut response = client.get("/resume").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("Search is resumed.".into()));
    let mut response = client.get("/close").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("Solve handle closed.".into()));
    let response = client.get("/statistics").dispatch();
    assert_eq!(response.status(), Status::Ok);
    // assert_eq!(
    //     response.body_string(),
    //     Some("InternalError: Solver::solve failed! No control object.".into())
    // );
}
#[test]
fn test_register_dl_theory() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/register_dl_theory").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.body_string().unwrap();
    let data: Value = serde_json::from_str(&data).unwrap();
    assert_eq!(data["type"], "InternalError");
    assert_eq!(
        &data["msg"],
        "Solver::register_dl_theory failed! No control object."
    );
}
#[test]
fn test_add() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.post("/add").body("body.").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.body_string().unwrap();
    let data: Value = serde_json::from_str(&data).unwrap();
    assert_eq!(data["type"], "InternalError");
    assert_eq!(&data["msg"], "Solver::add failed! No control object.");
}
#[test]
fn test_ground() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/ground").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.body_string().unwrap();
    let data: Value = serde_json::from_str(&data).unwrap();
    assert_eq!(data["type"], "InternalError");
    assert_eq!(&data["msg"], "Solver::ground failed! No control object.");
}
#[test]
fn test_solve() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/solve").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.body_string().unwrap();
    let data: Value = serde_json::from_str(&data).unwrap();
    assert_eq!(data["type"], "InternalError");
    assert_eq!(&data["msg"], "Solver::solve failed! No control object.");
}
#[test]
fn test_model() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/model").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.body_string().unwrap();
    let data: Value = serde_json::from_str(&data).unwrap();
    assert_eq!(data["type"], "InternalError");
    assert_eq!(
        data["msg"],
        Value::String("Solver::model failed! No SolveHandle.".to_string())
    );
}
#[test]
fn test_resume() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/resume").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.body_string().unwrap();
    let data: Value = serde_json::from_str(&data).unwrap();
    assert_eq!(data["type"], "InternalError");
    assert_eq!(
        data["msg"],
        Value::String("Solver::resume failed! No SolveHandle.".to_string())
    );
}
#[test]
fn test_close() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/close").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.body_string().unwrap();
    let data: Value = serde_json::from_str(&data).unwrap();
    assert_eq!(data["type"], "InternalError");
    assert_eq!(
        data["msg"],
        Value::String("Solver::close failed! No SolveHandle.".to_string())
    );
}
#[test]
fn test_statistics() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/statistics").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let data = response.body_string().unwrap();
    let data: Value = serde_json::from_str(&data).unwrap();
    assert_eq!(data["type"], "InternalError");
    assert_eq!(
        &data["msg"],
        "Solver::statistics failed! No control object."
    );
}
