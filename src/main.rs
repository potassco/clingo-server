#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod utils;
use clingo::{Part, SolveMode};
use rocket::response::Stream;
use rocket::{Data, State};
use std::io::Read;
use std::sync::{Arc, Mutex};
use utils::{write_model, ModelStream, RequestId, ServerError, Solver};

#[get("/")]
fn index(id: &RequestId) -> String {
    format!("This is request #{}.", id.0)
}

#[get("/create")]
fn create(state: State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock().unwrap();
    solver.create(vec![String::from("0")])?;
    Ok("Created clingo Solver.".to_string())
}
#[post("/add", format = "plain", data = "<data>")]
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
fn model(state: State<Arc<Mutex<Solver>>>) -> Result<Stream<ModelStream>, ServerError> {
    let mut solver = state.lock().unwrap();
    let mut buf = vec![];
    match solver.model() {
        // write the model
        Ok(Some(model)) => {
            write_model(model, &mut buf)?;
            solver.resume().unwrap();
            Ok(Stream::from(ModelStream { buf }))
        }
        // stop if there are no more models
        Ok(None) => Ok(Stream::from(ModelStream { buf })),
        Err(e) => Err(e),
    }
}
#[get("/close")]
fn close(state: State<Arc<Mutex<Solver>>>) -> Result<String, ServerError> {
    let mut solver = state.lock().unwrap();
    solver.close()?;
    Ok("Solve handle closed.".to_string())
}

fn main() {
    let state: Arc<Mutex<Solver>> = Arc::new(Mutex::new(Solver::Control(None)));
    rocket::ignite()
        .manage(state)
        .mount(
            "/",
            routes![index, create, add, ground, solve, model, close],
        )
        .launch();
}
