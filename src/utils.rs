use clingo::{ast, dl_theory::DLTheory};
use clingo::{dl_theory::DLTheoryAssignment, theory::Theory};
use clingo::{
    ClingoError, Configuration, ConfigurationType, Control, Id, Literal, Model, Part, ShowType,
    SolveHandle, SolveMode, Statistics, StatisticsType,
};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::cmp;
use std::io;
use std::io::{Read, Write};
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

                    // let conf = ctl.configuration_mut().unwrap();
                    // let root_key = conf.root().unwrap();

                    // print_configuration(conf, root_key, 0);

                    *self = Solver::DLControl(Some((ctl, dl_theory)));
                    Ok(())
                }
            },
            Solver::DLControl(Some(_)) => Err(ServerError::InternalError {
                msg: "Solver::register_dl_theory failed! DLTheory already registered.",
            }),
        }
    }
    pub fn statistics(&mut self) -> Result<Vec<u8>, ServerError> {
        match self {
            Solver::SolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::statistics() failed! Solving has already started.",
            }),
            Solver::DLSolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::statistics() failed! DLSolving has already started.",
            }),
            Solver::Control(ctl) => match ctl.take() {
                None => Err(ServerError::InternalError {
                    msg: "Solver::statistics() failed! No Control object.",
                }),
                Some(ctl) => {
                    let stats = ctl.statistics()?;
                    let stats_key = stats.root().unwrap();
                    let mut buf = Vec::new();
                    write_statistics(&mut buf, stats, stats_key, 0);
                    Ok(buf)
                }
            },
            Solver::DLControl(ctl) => match ctl.take() {
                None => Err(ServerError::InternalError {
                    msg: "Solver::solve() failed! No Control object.",
                }),
                Some((ctl, _dl_theory)) => {
                    let stats = ctl.statistics()?;
                    let stats_key = stats.root().unwrap();
                    let mut buf = Vec::new();
                    write_statistics(&mut buf, stats, stats_key, 0);
                    Ok(buf)
                }
            },
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
                                write_dl_theory_assignment(
                                    dl_theory.assignment(model.thread_id().unwrap()),
                                    &mut buf,
                                )?;
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
                Some((handle, _dl_theory)) => {
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

    for symbol in atoms {
        writeln!(out, "{}", symbol)?;
    }
    Ok(())
}
pub fn write_dl_theory_assignment(
    dl_theory_assignment: DLTheoryAssignment,
    mut out: impl io::Write,
) -> Result<(), io::Error> {
    for (symbol, theory_value) in dl_theory_assignment {
        writeln!(out, "{}={}", symbol, theory_value)?;
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
            match event {
                clingo::SolveEvent::Model(model) => {
                    eprintln!("model event goon: true");
                    true
                }
                clingo::SolveEvent::Statistics { step, akku } => {
                    eprintln!("statistics event goon: true");
                    true
                }
                _ => true,
            }
        } else {
            match event {
                clingo::SolveEvent::Model(model) => self.theory.on_model(model),
                clingo::SolveEvent::Statistics { step, akku } => {
                    eprintln!("statistics event");
                    self.theory.on_statistics(step, akku)
                }
                _ => true,
            }
        }
    }
}

fn write_prefix(buf: &mut impl Write, depth: u8) {
    writeln!(buf).unwrap();
    for _ in 0..depth {
        write!(buf, "  ").unwrap();
    }
}

// recursively write the statistics object
fn write_statistics(buf: &mut impl Write, stats: &Statistics, key: u64, depth: u8) {
    // get the type of an entry and switch over its various values
    let statistics_type = stats.statistics_type(key).unwrap();
    match statistics_type {
        // write values
        StatisticsType::Value => {
            let value = stats
                .value_get(key)
                .expect("Failed to retrieve statistics value.");
            write!(buf, " {}", value).unwrap();
        }

        // write arrays
        StatisticsType::Array => {
            // loop over array elements
            let size = stats
                .array_size(key)
                .expect("Failed to retrieve statistics array size.");
            for i in 0..size {
                // write array offset (with prefix for readability)
                let subkey = stats
                    .array_at(key, i)
                    .expect("Failed to retrieve statistics array.");
                write_prefix(buf, depth);
                write!(buf, "{} zu:", i).unwrap();

                // recursively write subentry
                write_statistics(buf, stats, subkey, depth + 1);
            }
        }

        // write maps
        StatisticsType::Map => {
            // loop over map elements
            let size = stats.map_size(key).unwrap();
            for i in 0..size {
                // get and write map name (with prefix for readability)
                let name = stats.map_subkey_name(key, i).unwrap();
                let subkey = stats.map_at(key, name).unwrap();
                write_prefix(buf, depth);
                write!(buf, "{}:", name).unwrap();

                // recursively write subentry
                write_statistics(buf, stats, subkey, depth + 1);
            }
        }

        // this case won't occur if the statistics are traversed like this
        StatisticsType::Empty => {
            writeln!(buf, "StatisticsType::Empty").unwrap();
        }
    }
}

fn print_prefix(depth: u8) {
    println!();
    for _ in 0..depth {
        print!("  ");
    }
}

// recursively print the configuartion object
fn print_configuration(conf: &Configuration, key: Id, depth: u8) {
    // get the type of an entry and switch over its various values
    let configuration_type = conf.configuration_type(key).unwrap();
    match configuration_type {
        // print values
        ConfigurationType::VALUE => {
            let value = conf
                .value_get(key)
                .expect("Failed to retrieve statistics value.");

            println!("{}", value);
        }

        // print arrays
        ConfigurationType::ARRAY => {
            // loop over array elements
            let size = conf
                .array_size(key)
                .expect("Failed to retrieve statistics array size.");
            for i in 0..size {
                // print array offset (with prefix for readability)
                let subkey = conf
                    .array_at(key, i)
                    .expect("Failed to retrieve statistics array.");
                print_prefix(depth);
                print!("{}:", i);

                // recursively print subentry
                print_configuration(conf, subkey, depth + 1);
            }
        }

        // print maps
        ConfigurationType::MAP => {
            // loop over map elements
            let size = conf.map_size(key).unwrap();
            for i in 0..size {
                // get and print map name (with prefix for readability)
                let name = conf.map_subkey_name(key, i).unwrap();
                let subkey = conf.map_at(key, name).unwrap();
                print_prefix(depth);
                print!("{}:", name);

                // recursively print subentry
                print_configuration(conf, subkey, depth + 1);
            }
        }

        // this case won't occur if the configuration are traversed like this
        _ => {
            let bla = conf.value_get(key).unwrap();
            print!(" {}", bla);
            // println!("Unknown ConfigurationType");
        }
    }
}
