use clingo::{
    ast, control, dl_theory::DLTheory, ClingoError, Configuration, ConfigurationType, Control, Id,
    Literal, Model, Part, ShowType, SolveHandle, SolveHandleWithEventHandler, SolveMode,
    Statistics, StatisticsType, Symbol, SymbolicAtoms,
};
type DLSolveHandle = SolveHandleWithEventHandler<DLEventHandler>;
use clingo::theory::Theory;
use rocket::response::{self, Responder};
use rocket_contrib::json::Json;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::cell::RefCell;
use std::cmp;
use std::io;
use std::io::{Read, Write};
use std::rc::Rc;
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
impl Responder<'static> for ServerError {
    fn respond_to(self, request: &Request<'_>) -> response::Result<'static> {
        let json = Json(self);
        json.respond_to(request)
    }
}
impl Serialize for ServerError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ServerError", 2)?;
        match self {
            ServerError::ClingoError(e) => {
                s.serialize_field("type", "ClingoError")?;
                s.serialize_field("msg", &format!("{}", e))?;
            }
            ServerError::IOError(e) => {
                s.serialize_field("type", "IoError")?;
                s.serialize_field("msg", &format!("{}", e))?;
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

use serde::ser::SerializeMap;
use serde::ser::SerializeSeq;
#[derive(Debug)]
pub enum StatisticsResult {
    Value(f64),
    Array(Vec<StatisticsResult>),
    Map(Vec<(String, StatisticsResult)>),
    Empty,
}
impl Serialize for StatisticsResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Array(array) => {
                let mut seq = serializer.serialize_seq(Some(array.len()))?;
                for e in array {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
            Self::Map(array) => {
                let mut map = serializer.serialize_map(Some(array.len()))?;
                for (k, v) in array {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Self::Value(value) => serializer.serialize_f64(*value),
            Self::Empty => serializer.serialize_unit(),
        }
    }
}
#[derive(Debug)]
pub enum ConfigurationResult {
    Value(String),
    Array(Vec<ConfigurationResult>),
    Map(Vec<(String, ConfigurationResult)>),
}
impl Serialize for ConfigurationResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Array(array) => {
                let mut seq = serializer.serialize_seq(Some(array.len()))?;
                for e in array {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
            Self::Map(array) => {
                let mut map = serializer.serialize_map(Some(array.len()))?;
                for (k, v) in array {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Self::Value(value) => serializer.serialize_str(value),
        }
    }
}
pub struct DLEventHandler {
    theory: Rc<RefCell<DLTheory>>,
}
impl clingo::SolveEventHandler for DLEventHandler {
    fn on_solve_event(&mut self, event: clingo::SolveEvent<'_>, goon: &mut bool) -> bool {
        match event {
            clingo::SolveEvent::Model(model) => self.theory.borrow_mut().on_model(model),
            clingo::SolveEvent::Statistics { step, akku } => {
                self.theory.borrow_mut().on_statistics(step, akku)
            }
            _ => true,
        }
    }
}
pub enum Solver {
    None,
    Control(ControlWrapper),
    SolveHandle(SolveHandleWrapper),
}
pub enum ControlWrapper {
    DLTheory(Control, Rc<RefCell<DLTheory>>),
    NoTheory(Control),
}
impl ControlWrapper {
    fn configuration<'a>(&'a mut self) -> Result<&Configuration, ClingoError> {
        match self {
            ControlWrapper::DLTheory(ctl, _) => ctl.configuration(),
            ControlWrapper::NoTheory(ctl) => ctl.configuration(),
        }
    }
    fn configuration_mut<'a>(&'a mut self) -> Result<&mut Configuration, ClingoError> {
        match self {
            ControlWrapper::DLTheory(ctl, _) => ctl.configuration_mut(),
            ControlWrapper::NoTheory(ctl) => ctl.configuration_mut(),
        }
    }
    fn statistics<'a>(&'a mut self) -> Result<&Statistics, ClingoError> {
        match self {
            ControlWrapper::DLTheory(ctl, _) => ctl.statistics(),
            ControlWrapper::NoTheory(ctl) => ctl.statistics(),
        }
    }

    pub fn symbolic_atoms<'a>(&self) -> Result<&'a SymbolicAtoms, ClingoError> {
        match self {
            ControlWrapper::DLTheory(ctl, _) => ctl.symbolic_atoms(),
            ControlWrapper::NoTheory(ctl) => ctl.symbolic_atoms(),
        }
    }
}
pub enum SolveHandleWrapper {
    DLTheory(DLSolveHandle, Rc<RefCell<DLTheory>>),
    NoTheory(SolveHandle),
}
impl Default for Solver {
    fn default() -> Self {
        Solver::None
    }
}
use std::mem;
impl Solver {
    pub fn take(&mut self) -> Solver {
        mem::take(self)
    }
}
unsafe impl Send for Solver {}
impl Solver {
    pub fn create(&mut self, arguments: std::vec::Vec<String>) -> Result<(), ServerError> {
        match self {
            Solver::None => {
                *self = Solver::Control(ControlWrapper::NoTheory(control(arguments)?));
                Ok(())
            }
            Solver::SolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::create failed! Solver still running!",
            }),
            Solver::Control(_) => {
                let mut ctl = control(arguments)?;
                *self = Solver::Control(ControlWrapper::NoTheory(ctl));
                Ok(())
            }
        }
    }
    pub fn close(&mut self) -> Result<(), ServerError> {
        let x = self.take();
        match x {
            Solver::None => {
                return Err(ServerError::InternalError {
                    msg: "Solver::close failed! Solver is not running.",
                })
            }
            Solver::Control(_) => {
                *self = x;
                return Err(ServerError::InternalError {
                    msg: "Solver::close failed! Solver is not running.",
                });
            }
            Solver::SolveHandle(SolveHandleWrapper::DLTheory(handle, dl_theory)) => {
                *self = Solver::Control(ControlWrapper::DLTheory(handle.close()?, dl_theory));
            }
            Solver::SolveHandle(SolveHandleWrapper::NoTheory(handle)) => {
                *self = Solver::Control(ControlWrapper::NoTheory(handle.close()?));
            }
        };
        Ok(())
    }
    pub fn solve(&mut self, mode: SolveMode, assumptions: &[Literal]) -> Result<(), ServerError> {
        let x = self.take();
        match x {
            Solver::None => {
                return Err(ServerError::InternalError {
                    msg: "Solver::solve failed! No control object.",
                })
            }
            Solver::SolveHandle(_) => {
                *self = x;
                return Err(ServerError::InternalError {
                    msg: "Solver::solve failed! DLSolving has already started.",
                });
            }
            Solver::Control(ControlWrapper::DLTheory(ctl, dl_theory)) => {
                let on_model = DLEventHandler {
                    theory: dl_theory.clone(),
                };

                *self = Solver::SolveHandle(SolveHandleWrapper::DLTheory(
                    ctl.solve_with_event_handler(mode, assumptions, on_model)?,
                    dl_theory,
                ));
            }
            _ => eprintln!("Unimplemented TheoryVariant."),
        };
        Ok(())
    }
    pub fn add(
        &mut self,
        name: &str,
        parameters: &[&str],
        program: &str,
    ) -> Result<(), ServerError> {
        match self {
            Solver::None => {
                return Err(ServerError::InternalError {
                    msg: "Solver::add failed! No control object.",
                })
            }
            Solver::SolveHandle(_) => {
                return Err(ServerError::InternalError {
                    msg: "Solver::add failed! Solver has been already started.",
                })
            }
            Solver::Control(ControlWrapper::DLTheory(ctl, dl_theory)) => {
                let mut rewriter = Rewriter {
                    control: ctl,
                    theory: dl_theory.clone(),
                };
                // rewrite the program
                clingo::ast::parse_string_with_statement_handler(program, &mut rewriter)?;
            }
            _ => eprintln!("Unimplemented TheoryVariant."),
        };
        Ok(())
    }
    pub fn ground(&mut self, parts: &[Part]) -> Result<(), ServerError> {
        match self {
            Solver::None => {
                return Err(ServerError::InternalError {
                    msg: "Solver::ground failed! No control object.",
                })
            }
            Solver::SolveHandle(_) => {
                return Err(ServerError::InternalError {
                    msg: "Solver::ground failed! Solver has been already started.",
                })
            }
            Solver::Control(ControlWrapper::DLTheory(ctl, dl_theory)) => {
                ctl.ground(parts)?;
                dl_theory.borrow_mut().prepare(ctl);
            }
            _ => eprintln!("Unimplemented TheoryVariant."),
        };
        Ok(())
    }
    pub fn register_dl_theory(&mut self) -> Result<(), ServerError> {
        let x = self.take();
        match x {
            Solver::None => {
                return Err(ServerError::InternalError {
                    msg: "Solver::register_dl_theory failed! No control object.",
                })
            }
            Solver::SolveHandle(_) => {
                *self = x;
                return Err(ServerError::InternalError {
                    msg: "Solver::register_dl_theory failed! Solver has been already started.",
                });
            }
            Solver::Control(ControlWrapper::DLTheory(mut ctl, _)) => {
                let mut dl_theory = DLTheory::create();
                dl_theory.register(&mut ctl);
                *self = Solver::Control(ControlWrapper::DLTheory(
                    ctl,
                    Rc::new(RefCell::new(dl_theory)),
                ));
            }
            Solver::Control(ControlWrapper::NoTheory(mut ctl)) => {
                let mut dl_theory = DLTheory::create();
                dl_theory.register(&mut ctl);
                *self = Solver::Control(ControlWrapper::DLTheory(
                    ctl,
                    Rc::new(RefCell::new(dl_theory)),
                ));
            }
        };
        Ok(())
    }
    pub fn statistics(&mut self) -> Result<StatisticsResult, ServerError> {
        match self {
            Solver::None => Err(ServerError::InternalError {
                msg: "Solver::statistics failed! No control object.",
            }),
            Solver::SolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::statistics failed! Solving has already started.",
            }),
            Solver::Control(ctl) => {
                let stats = ctl.statistics()?;
                let root_key = stats.root()?;
                let stats_result = parse_statistics(stats, root_key)?;
                Ok(stats_result)
            }
        }
    }
    pub fn configuration(&mut self) -> Result<ConfigurationResult, ServerError> {
        match self {
            Solver::None => Err(ServerError::InternalError {
                msg: "Solver::configuration failed! No control object.",
            }),
            Solver::SolveHandle(__) => Err(ServerError::InternalError {
                msg: "Solver::configuration failed! Solving has already started.",
            }),
            Solver::Control(ctl) => {
                let conf = ctl.configuration()?;
                let root_key = conf.root()?;
                let conf_result = parse_configuration(conf, root_key)?;
                Ok(conf_result)
            }
        }
    }
    pub fn set_configuration(
        &mut self,
        new_conf: &ConfigurationResult,
    ) -> Result<ConfigurationResult, ServerError> {
        match self {
            Solver::None => Err(ServerError::InternalError {
                msg: "Solver::set_configuration failed! No control object.",
            }),
            Solver::SolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::set_configuration failed! Solving has already started.",
            }),
            Solver::Control(ctl) => {
                let conf = ctl.configuration_mut()?;
                let root_key = conf.root()?;
                __set_conf(conf, new_conf, root_key)?;
                let conf_result = parse_configuration(conf, root_key)?;
                Ok(conf_result)
            }
        }
    }
    pub fn solve_with_assumptions(
        &mut self,
        assumptions: &[(clingo::Symbol, bool)],
    ) -> Result<(), ServerError> {
        match self {
            Solver::None => Err(ServerError::InternalError {
                msg: "Solver::solve_with_assumptions failed! No control object.",
            }),
            Solver::SolveHandle(_) => Err(ServerError::InternalError {
                msg: "Solver::solve_with_assumptions failed! Solving has already started.",
            }),
            Solver::Control(ctl) => {
                // get the program literal corresponding to the external atom
                let atoms = ctl.symbolic_atoms()?;
                let mut atm_it = atoms.iter()?;

                let mut assumption_literals = Vec::with_capacity(assumptions.len());
                for (sym, sign) in assumptions {
                    if let Some(item) = atm_it.find(|e| e.symbol().unwrap() == *sym) {
                        let mut lit = item.literal()?;
                        if !*sign {
                            lit = lit.negate();
                        }
                        assumption_literals.push(lit)
                    } else {
                        return Err(ServerError::InternalError {
                            msg: "Solver::solve_with_assumptions failed! \
                                  The assumptions contain a literal that is not defined in the logic program.",
                        });
                    }
                }
                self.solve(SolveMode::ASYNC | SolveMode::YIELD, &assumption_literals)
            }
        }
    }
    pub fn model(&mut self) -> Result<ModelResult, ServerError> {
        match self {
            Solver::None => Err(ServerError::InternalError {
                msg: "Solver::model failed! No SolveHandle.",
            }),
            Solver::Control(_) => Err(ServerError::InternalError {
                msg: "Solver::model failed! Solving has not yet started.",
            }),
            Solver::SolveHandle(SolveHandleWrapper::DLTheory(handle, dl_theory)) => {
                if handle.wait(0.0) {
                    match handle.model_mut() {
                        Ok(Some(model)) => {
                            // dl_theory.on_model(model);
                            let mut buf = vec![];
                            write_model(model, &mut buf)?;
                            // TODO rewrite write_dl_theory_assignment to use boxed iterator
                            write_dl_theory_assignment(
                                dl_theory.borrow_mut().assignment(model.thread_id()?),
                                &mut buf,
                            )?;

                            Ok(ModelResult::Model(buf))
                        }
                        Ok(None) => Ok(ModelResult::Done),
                        Err(e) => Err(e.into()),
                    }
                } else {
                    Ok(ModelResult::Running)
                }
            }
            Solver::SolveHandle(SolveHandleWrapper::NoTheory(handle)) => {
                if handle.wait(0.0) {
                    match handle.model_mut() {
                        Ok(Some(model)) => {
                            let mut buf = vec![];
                            write_model(model, &mut buf)?;
                            Ok(ModelResult::Model(buf))
                        }
                        Ok(None) => Ok(ModelResult::Done),
                        Err(e) => Err(e.into()),
                    }
                } else {
                    Ok(ModelResult::Running)
                }
            }
        }
    }
    pub fn resume(&mut self) -> Result<(), ServerError> {
        match self {
            Solver::None => Err(ServerError::InternalError {
                msg: "Solver::resume failed! No SolveHandle.",
            }),
            Solver::Control(_) => Err(ServerError::InternalError {
                msg: "Solver::resume failed! Solver has not yet started.",
            }),
            Solver::SolveHandle(SolveHandleWrapper::DLTheory(handle, _)) => {
                handle.resume()?;
                Ok(())
            }
            Solver::SolveHandle(SolveHandleWrapper::NoTheory(handle)) => {
                handle.resume()?;
                Ok(())
            }
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
pub fn write_dl_theory_assignment<'a>(
    dl_theory_assignment: Box<dyn Iterator<Item = (Symbol, clingo::theory::TheoryValue)> + 'a>,
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
pub struct Rewriter<'a> {
    control: &'a mut Control,
    theory: Rc<RefCell<DLTheory>>,
}

impl<'a> clingo::ast::StatementHandler for Rewriter<'a> {
    fn on_statement(&mut self, stm: &ast::Statement) -> bool {
        let mut builder = ast::ProgramBuilder::from(self.control).unwrap();
        self.theory
            .borrow_mut()
            .rewrite_statement(stm, &mut builder)
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
// recursively parse the statistics object
fn parse_statistics(stats: &Statistics, key: u64) -> Result<StatisticsResult, ClingoError> {
    // get the type of an entry and switch over its various values
    let statistics_type = stats.statistics_type(key)?;
    match statistics_type {
        // parse values
        StatisticsType::Value => {
            let value = stats.value_get(key)?;
            Ok(StatisticsResult::Value(value))
        }

        // parse arrays
        StatisticsType::Array => {
            // loop over array elements
            let size = stats.array_size(key)?;
            let mut array = vec![];
            for i in 0..size {
                let subkey = stats.array_at(key, i)?;
                // recursively parse subentry
                let x = parse_statistics(stats, subkey)?;
                array.push(x);
            }
            Ok(StatisticsResult::Array(array))
        }

        // parse maps
        StatisticsType::Map => {
            // loop over map elements
            let size = stats.map_size(key)?;
            let mut array = vec![];
            for i in 0..size {
                let name = stats.map_subkey_name(key, i)?;
                let subkey = stats.map_at(key, name)?;
                // recursively parse subentry
                let elem = parse_statistics(stats, subkey)?;
                array.push((name.to_string(), elem));
            }
            Ok(StatisticsResult::Map(array))
        }

        // this case won't occur if the statistics are traversed like this
        StatisticsType::Empty => Ok(StatisticsResult::Empty),
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
    if configuration_type.contains(ConfigurationType::VALUE) {
        // print values

        let value = conf
            .value_get(key)
            .expect("Failed to retrieve statistics value.");

        print!("{}", value);
    } else if configuration_type.contains(ConfigurationType::ARRAY) {
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
            print!("{}: ", i);

            // recursively print subentry
            print_configuration(conf, subkey, depth + 1);
        }
    } else if configuration_type.contains(ConfigurationType::MAP) {
        // loop over map elements
        let size = conf.map_size(key).unwrap();
        for i in 0..size {
            // get and print map name (with prefix for readability)
            let name = conf.map_subkey_name(key, i).unwrap();
            let subkey = conf.map_at(key, name).unwrap();
            print_prefix(depth);
            print!("{}: ", name);

            // recursively print subentry
            print_configuration(conf, subkey, depth + 1);
        }
    } else {
        eprintln!("Unknown ConfigurationType");
        unreachable!()
    }
}
// recursively parse the configuration object
fn parse_configuration(conf: &Configuration, key: Id) -> Result<ConfigurationResult, ClingoError> {
    // get the type of an entry and switch over its various values
    let configuration_type = conf.configuration_type(key)?;
    if configuration_type.contains(ConfigurationType::VALUE) {
        let value = conf.value_get(key)?;
        Ok(ConfigurationResult::Value(value))
    } else if configuration_type.contains(ConfigurationType::ARRAY) {
        let size = conf.array_size(key)?;

        let mut array = Vec::with_capacity(size);
        for i in 0..size {
            let subkey = conf.array_at(key, i)?;
            // recursively parse subentry
            let elem = parse_configuration(conf, subkey)?;
            array.push(elem);
        }
        Ok(ConfigurationResult::Array(array))
    } else if configuration_type.contains(ConfigurationType::MAP) {
        // loop over map elements
        let size = conf.map_size(key)?;
        let mut array = Vec::with_capacity(size);
        for i in 0..size {
            let name = conf.map_subkey_name(key, i)?;
            let subkey = conf.map_at(key, name)?;
            // recursively parse subentry
            let elem = parse_configuration(conf, subkey)?;
            array.push((name.to_string(), elem));
        }
        Ok(ConfigurationResult::Map(array))
    } else {
        eprintln!("Unknown ConfigurationType ");
        unreachable!()
    }
}
fn __set_conf(
    conf: &mut Configuration,
    new_conf: &ConfigurationResult,
    key: Id,
) -> Result<(), ClingoError> {
    match new_conf {
        ConfigurationResult::Value(v) => {
            conf.value_set(key, v)?;
        }
        ConfigurationResult::Array(arr) => {
            for (i, e) in arr.iter().enumerate() {
                let subkey = conf.array_at(key, i)?;
                __set_conf(conf, e, subkey)?;
            }
        }
        ConfigurationResult::Map(m) => {
            for (name, c) in m {
                let subkey = conf.map_at(key, name)?;
                __set_conf(conf, c, subkey)?;
            }
        }
    };
    Ok(())
}
