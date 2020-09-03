#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use clingo::*;
use rocket::response::Stream;
use rocket::Data;
use std::fs::File;
use std::io::Write;
use std::io::{prelude::*, BufReader, Error, ErrorKind};

#[get("/")]
fn index() -> &'static str {
    "Hello, world! This is cl-server!"
}

#[post("/upload", format = "plain", data = "<data>")]
fn upload(data: Data) -> Result<String, std::io::Error> {
    data.stream_to_file("/tmp/upload.lp").map(|n| n.to_string())
}

fn write_model(model: &Model, mut out: impl Write) -> Result<(), std::io::Error> {
    // retrieve the symbols in the model
    let atoms = match model.symbols(ShowType::SHOWN) {
        Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        Ok(atoms) => atoms,
    };

    for atom in atoms {
        // retrieve and write the symbol's string
        let atom_string = match atom.to_string() {
            Err(e) => return Err(Error::new(ErrorKind::Other, e)),
            Ok(atom_string) => atom_string,
        };

        // println!("{}", atom_string);
        writeln!(out, "{}", atom_string)?;
    }
    Ok(())
}

#[get("/retrieve")]
fn retrieve() -> Result<Stream<File>, std::io::Error> {
    let mut ctl = match Control::new(vec![]) {
        Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        Ok(ctl) => ctl,
    };
    let in_file = File::open("/tmp/upload.lp")?;
    let mut reader = BufReader::new(in_file);

    let mut buf = String::new();
    while 0 < reader.read_line(&mut buf)? {}

    match ctl.add("base", &[], &buf) {
        Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        Ok(()) => {}
    }
    // ground the base part
    let part = match Part::new("base", &[]) {
        Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        Ok(part) => part,
    };
    let parts = vec![part];
    match ctl.ground(&parts) {
        Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        Ok(()) => {}
    }

    // solve
    let mut handle = match ctl.solve(SolveMode::YIELD, &[]) {
        Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        Ok(handle) => handle,
    };
    // loop over all models
    let mut out_file = File::create("/tmp/answer.json")?;
    loop {
        match handle.resume() {
            Err(e) => return Err(Error::new(ErrorKind::Other, e)),
            Ok(()) => {}
        }
        match handle.model() {
            // print the model
            Ok(Some(model)) => write_model(model, &mut out_file)?,
            // stop if there are no more models
            Ok(None) => break,
            Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        }
    }
    File::open("/tmp/answer.json").map(Stream::from)
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, upload, retrieve])
        .launch();
}
