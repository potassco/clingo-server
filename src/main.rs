#![feature(proc_macro_hygiene, decl_macro)]

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
use rocket::{Data, State};
use rocket_contrib::json::Json;
use std::io::Read;
use std::sync::{Arc, Mutex};
use utils::{ConfigurationResult, ModelResult, RequestId, ServerError, Solver, StatisticsResult};

#[cfg(test)]
mod test;

#[get("/")]
fn index(id: &RequestId) -> String {
    format!("This is request #{}.", id.0)
}

#[get("/create")]
fn create(state: State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state
        .lock()
        .map_err(|_| ServerError::InternalError { msg: "PoisonError" })?;
    solver.create(vec!["0".to_string()])?;
    Ok("Created clingo Solver.".to_string())
}
#[post("/add", data = "<data>")]
fn add(state: State<Arc<Mutex<Solver>>>, data: Data) -> Result<String, ServerError> {
    let mut solver = state
        .lock()
        .map_err(|_| ServerError::InternalError { msg: "PoisonError" })?;
    let mut buf = String::new();
    let mut ds = data.open();
    ds.read_to_string(&mut buf)?;
    solver.add("base", &[], &buf)?;
    Ok("Added data to Solver.".to_string())
}
#[post("/ground", format = "application/json", data = "<data>")]
fn ground(state: State<Arc<Mutex<Solver>>>, data: Data) -> Result<String, ServerError> {
    let mut solver = state
        .lock()
        .map_err(|_| ServerError::InternalError { msg: "PoisonError" })?;

    let mut buf = String::new();
    let mut ds = data.open();
    ds.read_to_string(&mut buf)?;
    let val = serde_json::from_str(&buf).map_err(|_| ServerError::InternalError {
        msg: "Could not parse json data",
    })?;

    let parts = json_to_parts(&val)?;
    // ground the parts
    solver.ground(&parts)?;
    Ok("Grounding.".to_string())
}

#[post("/assign_external", format = "application/json", data = "<data>")]
fn assign_external(state: State<Arc<Mutex<Solver>>>, data: Data) -> Result<String, ServerError> {
    let mut solver = state
        .lock()
        .map_err(|_| ServerError::InternalError { msg: "PoisonError" })?;

    let mut buf = String::new();
    let mut ds = data.open();
    ds.read_to_string(&mut buf)?;
    let val = serde_json::from_str(&buf).map_err(|_| ServerError::InternalError {
        msg: "Could not parse json data",
    })?;

    let assignment = json_to_assignment(&val)?;
    solver.assign_external(&assignment)?;
    Ok("External assigned.".to_string())
}
#[post("/release_external", format = "application/json", data = "<data>")]
fn release_external(state: State<Arc<Mutex<Solver>>>, data: Data) -> Result<String, ServerError> {
    let mut solver = state
        .lock()
        .map_err(|_| ServerError::InternalError { msg: "PoisonError" })?;

    let mut buf = String::new();
    let mut ds = data.open();
    ds.read_to_string(&mut buf)?;
    let val = serde_json::from_str(&buf).map_err(|_| ServerError::InternalError {
        msg: "Could not parse json data",
    })?;

    let symbol = json_to_symbol(&val)?;
    solver.release_external(&symbol)?;
    Ok("External released.".to_string())
}
#[get("/solve")]
fn solve(state: State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state
        .lock()
        .map_err(|_| ServerError::InternalError { msg: "PoisonError" })?;
    solver.solve(SolveMode::ASYNC | SolveMode::YIELD, &[])?;
    Ok("Solving.".to_string())
}
#[post(
    "/solve_with_assumptions",
    format = "application/json",
    data = "<data>"
)]
fn solve_with_assumptions(
    state: State<Arc<Mutex<Solver>>>,
    data: Data,
) -> Result<String, ServerError> {
    let mut solver = state
        .lock()
        .map_err(|_| ServerError::InternalError { msg: "PoisonError" })?;
    let mut buf = String::new();
    let mut ds = data.open();
    ds.read_to_string(&mut buf)?;
    let val = serde_json::from_str(&buf).map_err(|_| ServerError::InternalError {
        msg: "Could not parse json data",
    })?;

    let assumptions = json_to_assumptions(&val)?;
    solver.solve_with_assumptions(&assumptions)?;
    Ok("Solving with assumptions.".to_string())
}
#[get("/model")]
fn model(state: State<Arc<Mutex<Solver>>>) -> Result<Json<ModelResult>, ServerError> {
    let mut solver = state
        .lock()
        .map_err(|_| ServerError::InternalError { msg: "PoisonError" })?;
    match solver.model() {
        Ok(mr) => Ok(Json(mr)),
        Err(e) => Err(e),
    }
}
#[get("/resume")]
fn resume(state: State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state
        .lock()
        .map_err(|_| ServerError::InternalError { msg: "PoisonError" })?;
    solver.resume()?;
    Ok("Search is resumed.".to_string())
}
#[get("/close")]
fn close(state: State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state
        .lock()
        .map_err(|_| ServerError::InternalError { msg: "PoisonError" })?;
    solver.close()?;
    Ok("Solve handle closed.".to_string())
}
#[get("/register_dl_theory")]
fn register_dl_theory(state: State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state
        .lock()
        .map_err(|_| ServerError::InternalError { msg: "PoisonError" })?;
    solver.register_dl_theory()?;
    Ok("Difference logic theory registered.".to_string())
}
#[get("/statistics")]
fn statistics(state: State<Arc<Mutex<Solver>>>) -> Result<Json<StatisticsResult>, ServerError> {
    let mut solver = state
        .lock()
        .map_err(|_| ServerError::InternalError { msg: "PoisonError" })?;
    match solver.statistics() {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err(e),
    }
}
#[get("/configuration")]
fn configuration(
    state: State<Arc<Mutex<Solver>>>,
) -> Result<Json<ConfigurationResult>, ServerError> {
    let mut solver = state
        .lock()
        .map_err(|_| ServerError::InternalError { msg: "PoisonError" })?;
    match solver.configuration() {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err(e),
    }
}
#[post("/set_configuration", format = "application/json", data = "<data>")]
fn set_configuration(state: State<Arc<Mutex<Solver>>>, data: Data) -> Result<String, ServerError> {
    let mut solver = state
        .lock()
        .map_err(|_| ServerError::InternalError { msg: "PoisonError" })?;
    let mut buf = String::new();
    let mut ds = data.open();
    ds.read_to_string(&mut buf)?;
    let val = serde_json::from_str(&buf).map_err(|_| ServerError::InternalError {
        msg: "Could not parse json data",
    })?;

    let c = json_to_configuration_result(&val)?;
    solver.set_configuration(&c)?;
    Ok("Set configuration.".to_string())
}

fn rocket() -> rocket::Rocket {
    let state: Arc<Mutex<Solver>> = Arc::new(Mutex::new(Solver::None));
    rocket::ignite().manage(state).mount(
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
            register_dl_theory
        ],
    )
}

fn main() {
    rocket().launch();
}
