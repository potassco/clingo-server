#[macro_use]
extern crate rocket;
extern crate rocket_okapi;
#[macro_use]
extern crate serde_derive;

mod convert;
mod utils;
use clingo::SolveMode;
use convert::{
    json_to_assignment, json_to_assumptions, json_to_configuration_result, json_to_parts,
    json_to_symbol,
};
use parking_lot::Mutex;
use rocket::data::ToByteUnit;
use rocket::serde::json::Json;
use rocket::{Data, State};
use rocket_okapi::{openapi, openapi_get_routes};
use std::sync::Arc;
use utils::{ConfigurationResult, ErrorResponse, ModelResult, Solver, StatisticsResult};

use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};

#[cfg(test)]
mod test;

fn get_docs() -> SwaggerUIConfig {
    use rocket_okapi::settings::UrlObject;

    SwaggerUIConfig {
        url: "/my_resource/openapi.json".to_string(),
        urls: vec![UrlObject::new("My Resource", "/v1/company/openapi.json")],
        ..Default::default()
    }
}

// #[openapi]
// #[get("/")]
// fn index(id: RequestId) -> String {
//     format!("This is request #{}.", id.0)
// }
#[openapi]
#[get("/create")]
fn create(state: &State<Arc<Mutex<Solver>>>) -> Result<String, ErrorResponse> {
    let mut solver = state.lock();
    solver.create(vec!["0".to_string()])?;
    Ok("Created clingo Solver.".to_string())
}
#[openapi]
#[post("/add", data = "<data>")]
async fn add(state: &State<Arc<Mutex<Solver>>>, data: Data<'_>) -> Result<String, ErrorResponse> {
    let ds = data.open(512.kibibytes());
    let cap = ds.into_string().await?;

    state.lock().add("base", &[], &cap.into_inner())?;
    Ok("Added data to Solver.".to_string())
}
#[openapi]
#[post("/ground", data = "<data>")]
// #[post("/ground", format = "application/json", data = "<parts>")]
async fn ground(
    state: &State<Arc<Mutex<Solver>>>,
    data: Data<'_>,
) -> Result<String, ErrorResponse> {
    let ds = data.open(512.kibibytes());
    let cap = ds.into_string().await?;
    let val = serde_json::from_str(&cap.into_inner())
        .map_err(|e| ErrorResponse::InternalError(format!("Could not parse json data {}", e)))?;

    let parts = json_to_parts(&val)?;
    // ground the parts
    let mut solver = state.lock();
    solver.ground(&parts)?;
    Ok("Grounding.".to_string())
}
#[openapi]
#[post("/assign_external", data = "<data>")]
async fn assign_external(
    state: &State<Arc<Mutex<Solver>>>,
    data: Data<'_>,
) -> Result<String, ErrorResponse> {
    let ds = data.open(512.kibibytes());
    let cap = ds.into_string().await?;
    let val = serde_json::from_str(&cap.into_inner())
        .map_err(|e| ErrorResponse::InternalError(format!("Could not parse json data {}", e)))?;

    let assignment = json_to_assignment(&val)?;
    let mut solver = state.lock();
    solver.assign_external(&assignment)?;
    Ok("External assigned.".to_string())
}
#[openapi]
#[post("/release_external", data = "<data>")]
async fn release_external(
    state: &State<Arc<Mutex<Solver>>>,
    data: Data<'_>,
) -> Result<String, ErrorResponse> {
    let ds = data.open(512.kibibytes());
    let cap = ds.into_string().await?;
    let val = serde_json::from_str(&cap.into_inner())
        .map_err(|e| ErrorResponse::InternalError(format!("Could not parse json data {}", e)))?;

    let symbol = json_to_symbol(&val)?;
    let mut solver = state.lock();
    solver.release_external(&symbol)?;
    Ok("External released.".to_string())
}
#[openapi]
#[get("/solve")]
fn solve(state: &State<Arc<Mutex<Solver>>>) -> Result<String, ErrorResponse> {
    let mut solver = state.lock();
    solver.solve(SolveMode::ASYNC | SolveMode::YIELD, &[])?;
    Ok("Solving.".to_string())
}
#[openapi]
#[post("/solve_with_assumptions", data = "<data>")]
async fn solve_with_assumptions(
    state: &State<Arc<Mutex<Solver>>>,
    data: Data<'_>,
) -> Result<String, ErrorResponse> {
    let ds = data.open(512.kibibytes());
    let cap = ds.into_string().await?;
    let val = serde_json::from_str(&cap.into_inner())
        .map_err(|e| ErrorResponse::InternalError(format!("Could not parse json data {}", e)))?;

    let assumptions = json_to_assumptions(&val)?;
    let mut solver = state.lock();
    solver.solve_with_assumptions(&assumptions)?;
    Ok("Solving with assumptions.".to_string())
}
#[openapi]
#[get("/model")]
fn model(state: &State<Arc<Mutex<Solver>>>) -> Result<Json<ModelResult>, ErrorResponse> {
    let mut solver = state.lock();
    match solver.model() {
        Ok(mr) => Ok(Json(mr)),
        Err(e) => Err(e)?,
    }
}
#[openapi]
#[get("/resume")]
fn resume(state: &State<Arc<Mutex<Solver>>>) -> Result<String, ErrorResponse> {
    let mut solver = state.lock();
    solver.resume()?;
    Ok("Search is resumed.".to_string())
}
#[openapi]
#[get("/close")]
fn close(state: &State<Arc<Mutex<Solver>>>) -> Result<String, ErrorResponse> {
    let mut solver = state.lock();
    solver.close()?;
    Ok("Solve handle closed.".to_string())
}
#[openapi]
#[get("/register_dl_theory")]
fn register_dl_theory(state: &State<Arc<Mutex<Solver>>>) -> Result<String, ErrorResponse> {
    let mut solver = state.lock();
    solver.register_dl_theory()?;
    Ok("Difference logic theory registered.".to_string())
}
#[openapi]
#[get("/register_con_theory")]
fn register_con_theory(state: &State<Arc<Mutex<Solver>>>) -> Result<String, ErrorResponse> {
    let mut solver = state.lock();
    solver.register_con_theory()?;
    Ok("Clingcon theory registered.".to_string())
}
#[openapi(skip)]
#[get("/statistics")]
fn statistics(state: &State<Arc<Mutex<Solver>>>) -> Result<Json<StatisticsResult>, ErrorResponse> {
    let mut solver = state.lock();
    match solver.statistics() {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err(e)?,
    }
}
#[openapi(skip)]
#[get("/configuration")]
fn configuration(
    state: &State<Arc<Mutex<Solver>>>,
) -> Result<Json<ConfigurationResult>, ErrorResponse> {
    let mut solver = state.lock();
    match solver.configuration() {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err(e)?,
    }
}
#[openapi]
#[post("/set_configuration", data = "<data>")]
async fn set_configuration(
    state: &State<Arc<Mutex<Solver>>>,
    data: Data<'_>,
) -> Result<String, ErrorResponse> {
    let ds = data.open(512.kibibytes());
    let cap = ds.into_string().await?;
    let val = serde_json::from_str(&cap.into_inner())
        .map_err(|e| ErrorResponse::InternalError(format!("Could not parse json data {}", e)))?;

    let c = json_to_configuration_result(&val)?;
    let mut solver = state.lock();
    solver.set_configuration(&c)?;
    Ok("Set configuration.".to_string())
}
#[launch]
fn rocket() -> _ {
    let state: Arc<Mutex<Solver>> = Arc::new(Mutex::new(Solver::None));
    rocket::build()
        .manage(state)
        // .mount(
        //     "/",
        //     routes![
        //         // create,
        //         // add,
        //     ],
        // )
        .mount(
            "/",
            openapi_get_routes![
                // index,
                create,
                add,
                ground,
                assign_external,
                release_external,
                solve,
                model,
                resume,
                close,
                statistics,
                configuration,
                set_configuration,
                solve_with_assumptions,
                register_dl_theory,
                register_con_theory
            ],
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
}
