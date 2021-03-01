#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

mod utils;
use clingo::{Part, SolveMode};
use rocket::{Data, State};
use rocket_contrib::json::Json;
use std::io::Read;
use std::sync::{Arc, Mutex};
use utils::{ModelResult, RequestId, ServerError, Solver};

#[cfg(test)]
mod test;

#[get("/")]
fn index(id: &RequestId) -> String {
    format!("This is request #{}.", id.0)
}

#[get("/create")]
fn create(state: State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock().unwrap();
    solver.create(vec!["0".to_string()])?;
    Ok("Created clingo Solver.".to_string())
}
// #[post("/add", format = "plain", data = "<data>")]
#[post("/add", data = "<data>")]
fn add(state: State<Arc<Mutex<Solver>>>, data: Data) -> Result<String, ServerError> {
    let mut solver = state.lock().unwrap();
    let mut ds = data.open();
    let mut buf = String::new();
    ds.read_to_string(&mut buf)?;
    solver.add("base", &[], &buf)?;
    Ok("Added data to Solver.".to_string())
}
#[get("/ground")]
fn ground(state: State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock().unwrap();
    // ground the base part
    let part = match Part::new("base", &[]) {
        Err(_) => {
            return Err(ServerError::InternalError {
                msg: "NulError while trying to create base Part",
            })
        }
        Ok(part) => part,
    };
    let parts = vec![part];
    solver.ground(&parts)?;
    Ok("Grounding.".to_string())
}
#[get("/solve")]
fn solve(state: State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock().unwrap();
    solver.solve(SolveMode::ASYNC | SolveMode::YIELD, &[])?;
    Ok("Solver solving.".to_string())
}
#[get("/model")]
fn model(state: State<Arc<Mutex<Solver>>>) -> Result<Json<ModelResult>, ServerError> {
    let mut solver = state.lock().unwrap();
    match solver.model() {
        Ok(mr) => Ok(Json(mr)),
        Err(e) => Err(e),
    }
}
#[get("/resume")]
fn resume(state: State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock().unwrap();
    solver.resume()?;
    Ok("Search is resumed.".to_string())
}
#[get("/close")]
fn close(state: State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock().unwrap();
    solver.close()?;
    Ok("Solve handle closed.".to_string())
}
#[get("/register_dl_theory")]
fn register_dl_theory(state: State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock().unwrap();
    solver.register_dl_theory()?;
    Ok("Difference logic theory registered.".to_string())
}
#[get("/statistics")]
fn statistics(state: State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock().unwrap();
    match solver.statistics() {
        Ok(stats) => Ok(String::from_utf8(stats).expect("expected utf8 string")),
        Err(e) => Err(e),
    }
}

fn rocket() -> rocket::Rocket {
    let state: Arc<Mutex<Solver>> = Arc::new(Mutex::new(Solver::Control(None)));
    rocket::ignite().manage(state).mount(
        "/",
        routes![
            index,
            create,
            add,
            ground,
            solve,
            model,
            resume,
            close,
            statistics,
            register_dl_theory
        ],
    )
}

fn main() {
    rocket().launch();
}
