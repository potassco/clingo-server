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
    assert_eq!(body_string, Some("{\"Model\":[97,10]}".into()));
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
    //     Some("InternalError: Solver::solve failed! No Control object.".into())
    // );
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
    let mut response = client.post("/add").body("body.").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some("InternalError: Solver::add failed! No control object.".into())
    );
}
#[test]
fn test_ground() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/ground").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some("InternalError: Solver::ground failed! No Control object.".into())
    );
}
#[test]
fn test_solve() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/solve").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some("InternalError: Solver::solve failed! No Control object.".into())
    );
}
#[test]
fn test_model() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/model").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some("InternalError: Solver::model failed! Solving has not yet started.".into())
    );
}
#[test]
fn test_resume() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/resume").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some("InternalError: Solver::resume failed! Solver has not yet started.".into())
    );
}
#[test]
fn test_close() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/close").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some("InternalError: Solver::close failed! Solving has not yet started.".into())
    );
}
#[test]
fn test_statistics() {
    let client = Client::new(rocket()).unwrap();
    let mut response = client.get("/statistics").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some("InternalError: Solver::statistics failed! No Control object.".into())
    );
}
