#[macro_use]
extern crate rocket;
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
use std::sync::Arc;
use utils::{ConfigurationResult, ModelResult, RequestId, ServerError, Solver, StatisticsResult};

#[cfg(test)]
mod test;

#[get("/")]
fn index(id: &RequestId) -> String {
    format!("This is request #{}.", id.0)
}
#[get("/create")]
fn create(state: &State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock();
    solver.create(vec!["0".to_string()])?;
    Ok("Created clingo Solver.".to_string())
}
#[post("/add", data = "<data>")]
async fn add(state: &State<Arc<Mutex<Solver>>>, data: Data<'_>) -> Result<String, ServerError> {
    let ds = data.open(512.kibibytes());
    let cap = ds.into_string().await?;

    state.lock().add("base", &[], &cap.into_inner())?;
    Ok("Added data to Solver.".to_string())
}
#[post("/ground", format = "application/json", data = "<data>")]
async fn ground(state: &State<Arc<Mutex<Solver>>>, data: Data<'_>) -> Result<String, ServerError> {
    let ds = data.open(512.kibibytes());
    let cap = ds.into_string().await?;
    let val = serde_json::from_str(&cap.into_inner()).map_err(|_| ServerError::InternalError {
        msg: "Could not parse json data",
    })?;

    let parts = json_to_parts(&val)?;
    // ground the parts
    let mut solver = state.lock();
    solver.ground(&parts)?;
    Ok("Grounding.".to_string())
}
#[post("/assign_external", format = "application/json", data = "<data>")]
async fn assign_external(
    state: &State<Arc<Mutex<Solver>>>,
    data: Data<'_>,
) -> Result<String, ServerError> {
    let ds = data.open(512.kibibytes());
    let cap = ds.into_string().await?;
    let val = serde_json::from_str(&cap.into_inner()).map_err(|_| ServerError::InternalError {
        msg: "Could not parse json data",
    })?;

    let assignment = json_to_assignment(&val)?;
    let mut solver = state.lock();
    solver.assign_external(&assignment)?;
    Ok("External assigned.".to_string())
}
#[post("/release_external", format = "application/json", data = "<data>")]
async fn release_external(
    state: &State<Arc<Mutex<Solver>>>,
    data: Data<'_>,
) -> Result<String, ServerError> {
    let ds = data.open(512.kibibytes());
    let cap = ds.into_string().await?;
    let val = serde_json::from_str(&cap.into_inner()).map_err(|_| ServerError::InternalError {
        msg: "Could not parse json data",
    })?;

    let symbol = json_to_symbol(&val)?;
    let mut solver = state.lock();
    solver.release_external(&symbol)?;
    Ok("External released.".to_string())
}
#[get("/solve")]
fn solve(state: &State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock();
    solver.solve(SolveMode::ASYNC | SolveMode::YIELD, &[])?;
    Ok("Solving.".to_string())
}
#[post(
    "/solve_with_assumptions",
    format = "application/json",
    data = "<data>"
)]
async fn solve_with_assumptions(
    state: &State<Arc<Mutex<Solver>>>,
    data: Data<'_>,
) -> Result<String, ServerError> {
    let ds = data.open(512.kibibytes());
    let cap = ds.into_string().await?;
    let val = serde_json::from_str(&cap.into_inner()).map_err(|_| ServerError::InternalError {
        msg: "Could not parse json data",
    })?;

    let assumptions = json_to_assumptions(&val)?;
    let mut solver = state.lock();
    solver.solve_with_assumptions(&assumptions)?;
    Ok("Solving with assumptions.".to_string())
}
#[get("/model")]
fn model(state: &State<Arc<Mutex<Solver>>>) -> Result<Json<ModelResult>, ServerError> {
    let mut solver = state.lock();
    match solver.model() {
        Ok(mr) => Ok(Json(mr)),
        Err(e) => Err(e),
    }
}
#[get("/resume")]
fn resume(state: &State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock();
    solver.resume()?;
    Ok("Search is resumed.".to_string())
}
#[get("/close")]
fn close(state: &State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock();
    solver.close()?;
    Ok("Solve handle closed.".to_string())
}
#[get("/register_dl_theory")]
fn register_dl_theory(state: &State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock();
    solver.register_dl_theory()?;
    Ok("Difference logic theory registered.".to_string())
}
#[get("/register_con_theory")]
fn register_con_theory(state: &State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock();
    solver.register_con_theory()?;
    Ok("Clingcon theory registered.".to_string())
}
#[get("/statistics")]
fn statistics(state: &State<Arc<Mutex<Solver>>>) -> Result<Json<StatisticsResult>, ServerError> {
    let mut solver = state.lock();
    match solver.statistics() {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err(e),
    }
}
#[get("/configuration")]
fn configuration(
    state: &State<Arc<Mutex<Solver>>>,
) -> Result<Json<ConfigurationResult>, ServerError> {
    let mut solver = state.lock();
    match solver.configuration() {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err(e),
    }
}
#[post("/set_configuration", format = "application/json", data = "<data>")]
async fn set_configuration(
    state: &State<Arc<Mutex<Solver>>>,
    data: Data<'_>,
) -> Result<String, ServerError> {
    let ds = data.open(512.kibibytes());
    let cap = ds.into_string().await?;
    let val = serde_json::from_str(&cap.into_inner()).map_err(|_| ServerError::InternalError {
        msg: "Could not parse json data",
    })?;

    let c = json_to_configuration_result(&val)?;
    let mut solver = state.lock();
    solver.set_configuration(&c)?;
    Ok("Set configuration.".to_string())
}
#[launch]
fn rocket() -> _ {
    let state: Arc<Mutex<Solver>> = Arc::new(Mutex::new(Solver::None));
    rocket::build().manage(state).mount(
        "/",
        routes![
            index,
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
}
