use clingo::{ClingoError, Control, Literal, Model, Part, ShowType, SolveHandle, SolveMode};
use std::cmp;
use std::io;
use std::io::Read;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("ClingoError: ")]
    ClingoError(#[from] ClingoError),
    #[error("ioError: ")]
    IOError(#[from] io::Error),
    #[error("InternalError: {msg}")]
    InternalError { msg: &'static str },
}

pub enum Solver {
    Control(Option<Control>),
    SolveHandle(Option<SolveHandle>),
}
unsafe impl Send for Solver {}
impl Solver {
    pub fn create(&mut self, arguments: std::vec::Vec<String>) -> Result<(), ServerError> {
        match self {
            Solver::SolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::create() failed! Solver still running!",
            }),
            Solver::Control(_) => {
                *self = Solver::Control(Some(Control::new(arguments)?));
                Ok(())
            }
        }
    }
    pub fn close(&mut self) -> Result<(), ServerError> {
        match self {
            Solver::Control(_) => Err(ServerError::InternalError {
                msg: "Solver::close() failed! Solving has not yet started.",
            }),
            Solver::SolveHandle(handle) => match handle.take() {
                None => Err(ServerError::InternalError {
                    msg: "Solver::close() failed! No SolveHandle.",
                }),
                Some(handle) => {
                    *self = Solver::Control(Some(handle.close()?));
                    Ok(())
                }
            },
        }
    }
    pub fn solve(&mut self, mode: SolveMode, assumptions: &[Literal]) -> Result<(), ServerError> {
        match self {
            Solver::SolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::solve() failed! Solving has already started.",
            }),
            Solver::Control(ctl) => match ctl.take() {
                None => Err(ServerError::InternalError {
                    msg: "Solver::solve() failed! No Control object.",
                }),
                Some(ctl) => {
                    *self = Solver::SolveHandle(Some(ctl.solve(mode, assumptions)?));
                    Ok(())
                }
            },
        }
    }
    pub fn add(
        &mut self,
        name: &str,
        parameters: &[&str],
        program: &str,
    ) -> Result<(), ServerError> {
        match self {
            Solver::SolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::add failed! Solver has been already started.",
            }),
            Solver::Control(ctl) => match ctl.as_mut() {
                None => Err(ServerError::InternalError {
                    msg: "Solver::add failed! No control object.",
                }),
                Some(ctl) => {
                    ctl.add(name, parameters, program)?;
                    Ok(())
                }
            },
        }
    }
    pub fn ground(&mut self, parts: &[Part]) -> Result<(), ServerError> {
        match self {
            Solver::SolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::ground failed! Solver has been already started.",
            }),
            Solver::Control(ctl) => match ctl.as_mut() {
                None => Err(ServerError::InternalError {
                    msg: "Solver::ground failed! No Control object.",
                }),
                Some(ctl) => {
                    ctl.ground(parts)?;
                    Ok(())
                }
            },
        }
    }
    pub fn model(&mut self) -> Result<Option<&Model>, ServerError> {
        match self {
            Solver::Control(_) => Err(ServerError::InternalError {
                msg: "Solver::model failed! Solving has not yet started.",
            }),
            Solver::SolveHandle(handle) => match handle.as_mut() {
                None => Err(ServerError::InternalError {
                    msg: "Solver::model failed! No SolveHandle.",
                }),
                Some(handle) => Ok(handle.model()?),
            },
        }
    }
    pub fn resume(&mut self) -> Result<(), ServerError> {
        match self {
            Solver::Control(_) => Err(ServerError::InternalError {
                msg: "Solver::resume failed! Solver has not yet started.",
            }),
            Solver::SolveHandle(handle) => match handle.as_mut() {
                None => Err(ServerError::InternalError {
                    msg: "Solver::resume failed! No SolveHandle.",
                }),
                Some(handle) => {
                    handle.resume()?;
                    Ok(())
                }
            },
        }
    }
}

pub fn write_model(model: &Model, mut out: impl io::Write) -> Result<(), io::Error> {
    // retrieve the symbols in the model
    let atoms = match model.symbols(ShowType::SHOWN) {
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
        Ok(atoms) => atoms,
    };

    for atom in atoms {
        // retrieve and write the symbol's string
        let atom_string = match atom.to_string() {
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
            Ok(atom_string) => atom_string,
        };
        writeln!(out, "{}", atom_string)?;
    }
    Ok(())
}
pub struct ModelStream {
    pub buf: Vec<u8>,
}
impl Read for ModelStream {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = cmp::min(buf.len(), self.buf.len());
        let (a, b) = self.buf.split_at(amt);
        println!(
            "buf: {}, self.buf: {} amt': {}",
            buf.len(),
            self.buf.len(),
            amt
        );
        println!("buf': {:?}", String::from_utf8(self.buf.clone()));
        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }
        self.buf = b.to_vec();
        Ok(amt)
    }
}

use rocket::request::{self, FromRequest, Request};
use std::sync::atomic::{AtomicUsize, Ordering};
static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A type that represents a request's ID.
pub struct RequestId(pub usize);
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
