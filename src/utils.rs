use clingo::{dl_theory::DLTheoryAssignmentIterator, theory::Theory};
use clingo::{ast, dl_theory::DLTheory};
use clingo::{ClingoError, Control, Literal, Model, Part, ShowType, SolveHandle, SolveMode};
use serde::ser::{Serialize, SerializeStruct, Serializer};
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
impl Serialize for ServerError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ServerError", 2)?;
        match self {
            ServerError::ClingoError(c) => {
                s.serialize_field("type", "ClingoError")?;
                s.serialize_field("msg", "")?;
            }
            ServerError::IOError(c) => {
                s.serialize_field("type", "IoError")?;
                s.serialize_field("msg", "")?;
            }
            ServerError::InternalError { msg } => {
                s.serialize_field("type", "InternalError")?;
                s.serialize_field("msg", *msg)?;
            }
        };
        s.end()
    }
}
#[derive(Debug, Serialize)]
pub enum ModelResult {
    Running,
    Model(Vec<u8>),
    Done,
}

pub enum Solver {
    Control(Option<Control>),
    DLControl(Option<(Control, DLTheory)>),
    SolveHandle(Option<SolveHandle>),
    DLSolveHandle(Option<(SolveHandle, DLTheory)>),
}
unsafe impl Send for Solver {}
impl Solver {
    pub fn create(&mut self, arguments: std::vec::Vec<String>) -> Result<(), ServerError> {
        match self {
            Solver::SolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::create() failed! Solver still running!",
            }),
            Solver::DLSolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::create() failed! Solver still running!",
            }),
            Solver::Control(_) => {
                *self = Solver::Control(Some(Control::new(arguments)?));
                Ok(())
            }
            Solver::DLControl(_) => {
                let mut ctl = Control::new(arguments)?;
                let mut dl_theory = DLTheory::create();
                dl_theory.register(&mut ctl);
                *self = Solver::DLControl(Some((ctl, dl_theory)));
                Ok(())
            }
        }
    }
    pub fn close(&mut self) -> Result<(), ServerError> {
        match self {
            Solver::Control(_) => Err(ServerError::InternalError {
                msg: "Solver::close() failed! Solving has not yet started.",
            }),
            Solver::DLControl(_) => Err(ServerError::InternalError {
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
            Solver::DLSolveHandle(handle) => match handle.take() {
                None => Err(ServerError::InternalError {
                    msg: "Solver::close() failed! No DLSolveHandle.",
                }),
                Some((handle, dl_theory)) => {
                    *self = Solver::DLControl(Some((handle.close()?, dl_theory)));
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
            Solver::DLSolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::solve() failed! DLSolving has already started.",
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
            Solver::DLControl(ctl) => match ctl.take() {
                None => Err(ServerError::InternalError {
                    msg: "Solver::solve() failed! No Control object.",
                }),
                Some((ctl, mut dl_theory)) => {
                    let mut on_model = DLModelHandler {
                        theory: &mut dl_theory,
                    };
                    *self = Solver::DLSolveHandle(Some((
                        ctl.solve_with_event_handler(mode, assumptions, &mut on_model)?,
                        dl_theory,
                    )));
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
            Solver::DLSolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::add failed! DLSolver has been already started.",
            }),
            Solver::Control(None) => Err(ServerError::InternalError {
                msg: "Solver::add failed! No control object.",
            }),
            Solver::DLControl(None) => Err(ServerError::InternalError {
                msg: "Solver::add failed! No control object.",
            }),
            Solver::Control(Some(ctl)) => {
                ctl.add(name, parameters, program)?;
                Ok(())
            }
            Solver::DLControl(Some((ctl, dl_theory))) => {
                let mut rewriter = Rewriter {
                    control: ctl,
                    theory: dl_theory,
                };
                // rewrite the program
                clingo::parse_program(program, &mut rewriter)
                    .expect("Failed to parse logic program.");
                Ok(())
            }
        }
    }
    pub fn ground(&mut self, parts: &[Part]) -> Result<(), ServerError> {
        match self {
            Solver::SolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::ground failed! Solver has been already started.",
            }),
            Solver::DLSolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::ground failed! DLSolver has been already started.",
            }),
            Solver::Control(None) => Err(ServerError::InternalError {
                msg: "Solver::ground failed! No Control object.",
            }),
            Solver::DLControl(None) => Err(ServerError::InternalError {
                msg: "Solver::ground failed! No Control object.",
            }),
            Solver::Control(Some(ctl)) => {
                ctl.ground(parts)?;
                Ok(())
            }
            Solver::DLControl(Some((ctl, dl_theory))) => {
                ctl.ground(parts)?;
                dl_theory.prepare(ctl);
                Ok(())
            }
        }
    }
    pub fn register_dl_theory(&mut self) -> Result<(), ServerError> {
        match self {
            Solver::SolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::register_dl_theory failed! Solver has been already started.",
            }),
            Solver::DLSolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::register_dl_theory failed! DLSolver has been already started.",
            }),
            Solver::DLControl(None) => Err(ServerError::InternalError {
                msg: "Solver::register_dl_theory failed! No Control object.",
            }),
            Solver::Control(ctl) => match ctl.take() {
                None => Err(ServerError::InternalError {
                    msg: "Solver::register_dl_theory failed! No Control object.",
                }),
                Some(mut ctl) => {
                    let mut dl_theory = DLTheory::create();
                    dl_theory.register(&mut ctl);
                    *self = Solver::DLControl(Some((ctl, dl_theory)));
                    Ok(())
                }
            },
            Solver::DLControl(Some(_)) => Err(ServerError::InternalError {
                msg: "Solver::register_dl_theory failed! DLTheory already registered.",
            }),
        }
    }
    pub fn model(&mut self) -> Result<ModelResult, ServerError> {
        match self {
            Solver::Control(_) => Err(ServerError::InternalError {
                msg: "Solver::model failed! Solving has not yet started.",
            }),
            Solver::DLControl(_) => Err(ServerError::InternalError {
                msg: "Solver::model failed! Solving has not yet started.",
            }),
            Solver::SolveHandle(handle) => match handle.as_mut() {
                None => Err(ServerError::InternalError {
                    msg: "Solver::model failed! No SolveHandle.",
                }),
                Some(handle) => {
                    if handle.wait(0.0) {
                        match handle.model() {
                            Ok(Some(model)) => {
                                let mut buf = vec![];
                                write_model(model, &mut buf)?;
                                Ok(ModelResult::Model(buf))
                            }
                            Ok(None) => Ok(ModelResult::Done),
                            Err(e) => {
                                println!("{}", e);
                                Err(ServerError::InternalError {
                                    msg: "Solver::model failed! ClingoError.",
                                })
                            }
                        }
                    } else {
                        Ok(ModelResult::Running)
                    }
                }
            },
            Solver::DLSolveHandle(handle) => match handle.as_mut() {
                None => Err(ServerError::InternalError {
                    msg: "Solver::model failed! No DLSolveHandle.",
                }),
                Some((handle, dl_theory)) => {
                    if handle.wait(0.0) {
                        match handle.model() {
                            Ok(Some(model)) => {
                                let mut buf = vec![];
                                write_model(model, &mut buf)?;
                                write_dl_theory_assignment(dl_theory.assignment_iter(model.thread_id().unwrap()), &mut buf)?;
                                Ok(ModelResult::Model(buf))
                            }
                            Ok(None) => Ok(ModelResult::Done),
                            Err(e) => {
                                println!("{}", e);
                                Err(ServerError::InternalError {
                                    msg: "Solver::model failed! ClingoError.",
                                })
                            }
                        }
                    } else {
                        Ok(ModelResult::Running)
                    }
                }
            },
        }
    }
    pub fn resume(&mut self) -> Result<(), ServerError> {
        match self {
            Solver::Control(_) => Err(ServerError::InternalError {
                msg: "Solver::resume failed! Solver has not yet started.",
            }),
            Solver::DLControl(_) => Err(ServerError::InternalError {
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
            Solver::DLSolveHandle(handle) => match handle.as_mut() {
                None => Err(ServerError::InternalError {
                    msg: "Solver::resume failed! No DLSolveHandle.",
                }),
                Some((handle, dl_theory)) => {
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
pub fn write_dl_theory_assignment(dlta_iterator: DLTheoryAssignmentIterator, mut out: impl io::Write) -> Result<(), io::Error> {
    for theory_value in dlta_iterator {
        writeln!(out, "{:?}", theory_value)?;
    }
    Ok(())
}
struct ModelStream {
    buf: Vec<u8>,
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
pub struct Rewriter<'a, 'b> {
    control: &'a mut Control,
    theory: &'b mut DLTheory,
}

impl<'a, 'b> clingo::StatementHandler for Rewriter<'a, 'b> {
    fn on_statement(&mut self, stm: &ast::Statement) -> bool {
        let mut builder = ast::ProgramBuilder::from(self.control).unwrap();
        self.theory.rewrite_statement(stm, &mut builder)
    }
}
pub struct DLModelHandler<'a> {
    theory: &'a mut DLTheory,
}

impl<'a> clingo::SolveEventHandler for DLModelHandler<'a> {
    fn on_solve_event(&mut self, event: clingo::SolveEvent<'_>, goon: &mut bool) -> bool {
        if *goon {
            true
        } else {
            match event {
                clingo::SolveEvent::Model(model) => {
                    self.theory.on_model(model)
                }
                _ => true,
            }
        }
    }
}
