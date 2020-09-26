#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod solver;
use clingo::{Part, SolveMode};
use rocket::request::{self, FromRequest, Request};
use rocket::response::Stream;
use rocket::{Data, State};
use solver::{write_model, ClingoServerError, ModelStream, Solver};
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A type that represents a request's ID.
struct RequestId(pub usize);
/// Returns the current request's ID, assigning one only as necessary.
impl<'a, 'r> FromRequest<'a, 'r> for &'a RequestId {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        // The closure passed to `local_cache` will be executed at most once per
        // request: the first time the `RequestId` guard is used. If it is
        // requested again, `local_cache` will return the same value.
        request::Outcome::Success(
            request.local_cache(|| RequestId(ID_COUNTER.fetch_add(1, Ordering::Relaxed))),
        )
    }
}

#[get("/")]
fn index(id: &RequestId) -> String {
    format!("This is request #{}.", id.0)
}

#[get("/create")]
fn create(sh_ctl: State<Arc<Mutex<Solver>>>) -> Result<String, ClingoServerError> {
    let mut sh_ctl: MutexGuard<Solver> = sh_ctl.lock().unwrap();
    sh_ctl.create(vec![String::from("0")])?;
    Ok("Created clingo Solver.".to_string())
}
#[post("/add", format = "plain", data = "<data>")]
fn add(sh_ctl: State<Arc<Mutex<Solver>>>, data: Data) -> Result<String, ClingoServerError> {
    let mut sh_ctl: MutexGuard<Solver> = sh_ctl.lock().unwrap();
    let mut ds = data.open();
    let mut buf = String::new();
    ds.read_to_string(&mut buf)?;
    sh_ctl.add("base", &[], &buf)?;
    Ok("Added data to Solver.".to_string())
}
#[get("/ground")]
fn ground(sh_ctl: State<Arc<Mutex<Solver>>>) -> Result<String, ClingoServerError> {
    let mut sh_ctl: MutexGuard<Solver> = sh_ctl.lock().unwrap();
    // ground the base part
    let part = match Part::new("base", &[]) {
        Err(_) => {
            return Err(ClingoServerError::InternalError {
                msg: "NulError while trying to create base Part",
            })
        }
        Ok(part) => part,
    };
    let parts = vec![part];
    sh_ctl.ground(&parts)?;
    Ok("Grounding.".to_string())
}
#[get("/solve")]
fn solve(sh_ctl: State<Arc<Mutex<Solver>>>) -> Result<String, ClingoServerError> {
    let mut sh_ctl: MutexGuard<Solver> = sh_ctl.lock().unwrap();
    sh_ctl.solve(SolveMode::ASYNC | SolveMode::YIELD, &[])?;
    Ok("Solver solving.".to_string())
}
#[get("/model")]
fn model(sh_ctl: State<Arc<Mutex<Solver>>>) -> Result<Stream<ModelStream>, ClingoServerError> {
    let mut sh_ctl = sh_ctl.lock().unwrap();
    let mut buf = vec![];
    match sh_ctl.model() {
        // write the model
        Ok(Some(model)) => {
            write_model(model, &mut buf)?;
            sh_ctl.resume().unwrap();
            Ok(Stream::from(ModelStream { buf }))
        }
        // stop if there are no more models
        Ok(None) => Ok(Stream::from(ModelStream { buf })),
        Err(e) => Err(e),
    }
}
#[get("/close")]
fn close(sh_ctl: State<Arc<Mutex<Solver>>>) -> Result<String, ClingoServerError> {
    let mut sh_ctl: MutexGuard<Solver> = sh_ctl.lock().unwrap();
    sh_ctl.close()?;
    Ok("Solve handle closed.".to_string())
}

fn main() {
    let ctl: Arc<Mutex<Solver>> = Arc::new(Mutex::new(Solver::Control(None)));
    rocket::ignite()
        .manage(ctl)
        .mount(
            "/",
            routes![index, create, add, ground, solve, model, close],
        )
        .launch();
}
