use clingcon_plugin::ConTheory;
use clingo::{
    ast, control, ClingoError, Configuration, ConfigurationType, Control, Id, Model, Part,
    ShowType, SolveHandle, SolveHandleWithEventHandler, SolveMode, SolverLiteral, Statistics,
    StatisticsType, Symbol, SymbolicAtoms, TruthValue,
};
use clingo_dl_plugin::DLTheory;
type DLSolveHandle = SolveHandleWithEventHandler<DLEventHandler>;
type ConSolveHandle = SolveHandleWithEventHandler<ConEventHandler>;
use clingo::theory::Theory;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::cell::RefCell;
use std::cmp;
use std::fmt::Debug;
use std::io;
use std::io::Read;
use std::rc::Rc;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("ClingoError: ")]
    ClingoError(#[from] ClingoError),
    #[error("ioError: ")]
    IOError(#[from] io::Error),
    #[error("InternalError:")]
    InternalError(String),
}
impl<'r> Responder<'r, 'static> for ServerError {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
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
            ServerError::InternalError(msg) => {
                s.serialize_field("type", "InternalError")?;
                s.serialize_field("msg", msg)?;
            }
        };
        s.end()
    }
}
impl From<libloading::Error> for ServerError {
    fn from(e: libloading::Error) -> Self {
        ServerError::InternalError(format!("{}", e))
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
pub struct ConEventHandler {
    theory: Rc<RefCell<ConTheory>>,
}
impl clingo::SolveEventHandler for ConEventHandler {
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
    // DLTheory(Control, Rc<RefCell<DLTheory>>, Rc<RefCell<Library>>),
    ConTheory(Control, Rc<RefCell<ConTheory>>),
    NoTheory(Control),
}
impl ControlWrapper {
    fn configuration(&mut self) -> Result<&Configuration, ClingoError> {
        match self {
            ControlWrapper::DLTheory(ctl, _) => ctl.configuration(),
            ControlWrapper::ConTheory(ctl, _) => ctl.configuration(),
            ControlWrapper::NoTheory(ctl) => ctl.configuration(),
        }
    }
    fn configuration_mut(&mut self) -> Result<&mut Configuration, ClingoError> {
        match self {
            ControlWrapper::DLTheory(ctl, _) => ctl.configuration_mut(),
            ControlWrapper::ConTheory(ctl, _) => ctl.configuration_mut(),
            ControlWrapper::NoTheory(ctl) => ctl.configuration_mut(),
        }
    }
    fn statistics(&mut self) -> Result<&Statistics, ClingoError> {
        match self {
            ControlWrapper::DLTheory(ctl, _) => ctl.statistics(),
            ControlWrapper::ConTheory(ctl, _) => ctl.statistics(),
            ControlWrapper::NoTheory(ctl) => ctl.statistics(),
        }
    }

    pub fn symbolic_atoms<'a>(&self) -> Result<&'a SymbolicAtoms, ClingoError> {
        match self {
            ControlWrapper::DLTheory(ctl, _) => ctl.symbolic_atoms(),
            ControlWrapper::ConTheory(ctl, _) => ctl.symbolic_atoms(),
            ControlWrapper::NoTheory(ctl) => ctl.symbolic_atoms(),
        }
    }
    pub fn assign_external(
        &mut self,
        symbol: &Symbol,
        truth_value: &TruthValue,
    ) -> Result<(), ServerError> {
        // get the program literal corresponding to the external atom
        let atoms = self.symbolic_atoms()?;
        let mut atm_it = atoms.iter()?;
        let item = atm_it
            .find(|e| e.symbol().unwrap() == *symbol)
            .ok_or_else(|| ServerError::InternalError("external symbol not found".to_string()))?;
        let atm = item.literal()?;
        match self {
            ControlWrapper::DLTheory(ctl, _) => ctl.assign_external(atm, *truth_value),
            ControlWrapper::ConTheory(ctl, _) => ctl.assign_external(atm, *truth_value),
            ControlWrapper::NoTheory(ctl) => ctl.assign_external(atm, *truth_value),
        }?;
        Ok(())
    }
    pub fn release_external(&mut self, symbol: &Symbol) -> Result<(), ServerError> {
        // get the program literal corresponding to the external atom
        let atoms = self.symbolic_atoms()?;
        let mut atm_it = atoms.iter()?;
        let item = atm_it
            .find(|e| e.symbol().unwrap() == *symbol)
            .ok_or_else(|| ServerError::InternalError("external symbol not found".to_string()))?;
        let atm = item.literal()?;
        match self {
            ControlWrapper::DLTheory(ctl, _) => ctl.release_external(atm),
            ControlWrapper::ConTheory(ctl, _) => ctl.release_external(atm),
            ControlWrapper::NoTheory(ctl) => ctl.release_external(atm),
        }?;
        Ok(())
    }
}
pub enum SolveHandleWrapper {
    DLTheory(DLSolveHandle, Rc<RefCell<DLTheory>>),
    ConTheory(ConSolveHandle, Rc<RefCell<ConTheory>>),
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
            }
            Solver::SolveHandle(_) => {
                return Err(ServerError::InternalError(
                    "Solver::create failed! Solver still running!".to_string(),
                ))
            }
            Solver::Control(_) => {
                let ctl = control(arguments)?;
                *self = Solver::Control(ControlWrapper::NoTheory(ctl));
            }
        }
        Ok(())
    }
    pub fn register_dl_theory(&mut self) -> Result<(), ServerError> {
        let x = self.take();
        match x {
            Solver::None => {
                return Err(ServerError::InternalError(
                    "Solver::register_dl_theory failed! No control object.".to_string(),
                ))
            }
            Solver::SolveHandle(_) => {
                *self = x;
                return Err(ServerError::InternalError(
                    "Solver::register_dl_theory failed! Solver has been already started."
                        .to_string(),
                ));
            }
            Solver::Control(ControlWrapper::DLTheory(mut ctl, _)) => {
                let library_path = "clingodl";
                println!("Loading add() from {}", library_path);
                //Loads the library and gets a symbol (casting the function pointer so it has the desired signature)
                let lib = unsafe { libloading::Library::new(library_path) }?;
                let create: libloading::Symbol<unsafe fn() -> DLTheory> =
                    unsafe { lib.get(b"create") }?;
                let mut dl_theory = unsafe { create() };
                dl_theory.register(&mut ctl);

                *self = Solver::Control(ControlWrapper::DLTheory(
                    ctl,
                    Rc::new(RefCell::new(dl_theory)),
                ));
            }
            Solver::Control(ControlWrapper::ConTheory(mut ctl, _)) => {
                let library_path = "clingodl";
                println!("Loading add() from {}", library_path);
                //Loads the library and gets a symbol (casting the function pointer so it has the desired signature)
                let lib = unsafe { libloading::Library::new(library_path) }?;
                let create: libloading::Symbol<unsafe fn() -> DLTheory> =
                    unsafe { lib.get(b"create") }?;
                let mut dl_theory = unsafe { create() };
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
    pub fn register_con_theory(&mut self) -> Result<(), ServerError> {
        let x = self.take();
        match x {
            Solver::None => {
                return Err(ServerError::InternalError(
                    "Solver::register_con_theory failed! No control object.".to_string(),
                ))
            }
            Solver::SolveHandle(_) => {
                *self = x;
                return Err(ServerError::InternalError(
                    "Solver::register_con_theory failed! Solver has been already started."
                        .to_string(),
                ));
            }
            Solver::Control(ControlWrapper::DLTheory(mut ctl, _)) => {
                let library_path = "clingcon";
                println!("Loading add() from {}", library_path);
                //Loads the library and gets a symbol (casting the function pointer so it has the desired signature)
                let lib = unsafe { libloading::Library::new(library_path) }?;
                let create: libloading::Symbol<unsafe fn() -> ConTheory> =
                    unsafe { lib.get(b"create") }?;
                let mut con_theory = unsafe { create() };
                con_theory.register(&mut ctl);

                *self = Solver::Control(ControlWrapper::ConTheory(
                    ctl,
                    Rc::new(RefCell::new(con_theory)),
                ));
            }
            Solver::Control(ControlWrapper::ConTheory(mut ctl, _)) => {
                let library_path = "clingcon";
                println!("Loading add() from {}", library_path);
                //Loads the library and gets a symbol (casting the function pointer so it has the desired signature)

                let lib = unsafe { libloading::Library::new(library_path) }?;
                let create: libloading::Symbol<unsafe fn() -> ConTheory> =
                    unsafe { lib.get(b"create") }?;
                let mut con_theory = unsafe { create() };

                con_theory.register(&mut ctl);

                *self = Solver::Control(ControlWrapper::ConTheory(
                    ctl,
                    Rc::new(RefCell::new(con_theory)),
                ));
            }
            Solver::Control(ControlWrapper::NoTheory(mut ctl)) => {
                let mut con_theory = ConTheory::create();
                con_theory.register(&mut ctl);
                *self = Solver::Control(ControlWrapper::ConTheory(
                    ctl,
                    Rc::new(RefCell::new(con_theory)),
                ));
            }
        };
        Ok(())
    }
    pub fn close(&mut self) -> Result<(), ServerError> {
        let x = self.take();
        match x {
            Solver::None => {
                return Err(ServerError::InternalError(
                    "Solver::close failed! Solver is not running.".to_string(),
                ))
            }
            Solver::Control(_) => {
                *self = x;
                return Err(ServerError::InternalError(
                    "Solver::close failed! Solver is not running.".to_string(),
                ));
            }
            Solver::SolveHandle(SolveHandleWrapper::DLTheory(handle, dl_theory)) => {
                *self = Solver::Control(ControlWrapper::DLTheory(handle.close()?, dl_theory));
            }
            Solver::SolveHandle(SolveHandleWrapper::ConTheory(handle, con_theory)) => {
                *self = Solver::Control(ControlWrapper::ConTheory(handle.close()?, con_theory));
            }
            Solver::SolveHandle(SolveHandleWrapper::NoTheory(handle)) => {
                *self = Solver::Control(ControlWrapper::NoTheory(handle.close()?));
            }
        };
        Ok(())
    }
    pub fn solve(
        &mut self,
        mode: SolveMode,
        assumptions: &[SolverLiteral],
    ) -> Result<(), ServerError> {
        let x = self.take();
        match x {
            Solver::None => {
                return Err(ServerError::InternalError(
                    "Solver::solve failed! No control object.".to_string(),
                ))
            }
            Solver::SolveHandle(_) => {
                *self = x;
                return Err(ServerError::InternalError(
                    "Solver::solve failed! DLSolving has already started.".to_string(),
                ));
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
            Solver::Control(ControlWrapper::ConTheory(ctl, con_theory)) => {
                let on_model = ConEventHandler {
                    theory: con_theory.clone(),
                };

                *self = Solver::SolveHandle(SolveHandleWrapper::ConTheory(
                    ctl.solve_with_event_handler(mode, assumptions, on_model)?,
                    con_theory,
                ));
            }
            Solver::Control(ControlWrapper::NoTheory(ctl)) => {
                *self = Solver::SolveHandle(SolveHandleWrapper::NoTheory(
                    ctl.solve(mode, assumptions)?,
                ));
            }
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
                return Err(ServerError::InternalError(
                    "Solver::add failed! No control object.".to_string(),
                ))
            }
            Solver::SolveHandle(_) => {
                return Err(ServerError::InternalError(
                    "Solver::add failed! Solver has been already started.".to_string(),
                ))
            }
            Solver::Control(ControlWrapper::DLTheory(ctl, dl_theory)) => {
                let mut bld = ast::ProgramBuilder::from(ctl)?;
                let mut rewriter = Rewriter {
                    builder: &mut bld,
                    theory: dl_theory.clone(),
                };
                // rewrite the program
                clingo::ast::parse_string_with_statement_handler(program, &mut rewriter)?;
            }
            Solver::Control(ControlWrapper::ConTheory(ctl, con_theory)) => {
                let mut bld = ast::ProgramBuilder::from(ctl)?;
                let mut rewriter = ConRewriter {
                    builder: &mut bld,
                    theory: con_theory.clone(),
                };
                // rewrite the program
                clingo::ast::parse_string_with_statement_handler(program, &mut rewriter)?;
            }
            Solver::Control(ControlWrapper::NoTheory(ctl)) => {
                ctl.add(name, parameters, program)?;
            }
        };
        Ok(())
    }
    pub fn ground(&mut self, parts: &[Part]) -> Result<(), ServerError> {
        match self {
            Solver::None => {
                return Err(ServerError::InternalError(
                    "Solver::ground failed! No control object.".to_string(),
                ))
            }
            Solver::SolveHandle(_) => {
                return Err(ServerError::InternalError(
                    "Solver::ground failed! Solver has been already started.".to_string(),
                ))
            }
            Solver::Control(ControlWrapper::DLTheory(ctl, dl_theory)) => {
                ctl.ground(parts)?;
                dl_theory.borrow_mut().prepare(ctl);
            }
            Solver::Control(ControlWrapper::ConTheory(ctl, con_theory)) => {
                ctl.ground(parts)?;
                con_theory.borrow_mut().prepare(ctl);
            }
            Solver::Control(ControlWrapper::NoTheory(ctl)) => {
                ctl.ground(parts)?;
            }
        };
        Ok(())
    }
    pub fn assign_external(
        &mut self,
        (symbol, truth_value): &(clingo::Symbol, clingo::TruthValue),
    ) -> Result<(), ServerError> {
        match self {
            Solver::None => Err(ServerError::InternalError(
                "Solver::assign_external failed! No control object.".to_string(),
            )),
            Solver::SolveHandle(_) => Err(ServerError::InternalError(
                "Solver::assign_external failed! Solving has already started.".to_string(),
            )),
            Solver::Control(ctl) => ctl.assign_external(symbol, truth_value),
        }
    }
    pub fn release_external(&mut self, symbol: &Symbol) -> Result<(), ServerError> {
        match self {
            Solver::None => Err(ServerError::InternalError(
                "Solver::release_external failed! No control object.".to_string(),
            )),
            Solver::SolveHandle(_) => Err(ServerError::InternalError(
                "Solver::release_external failed! Solving has already started.".to_string(),
            )),
            Solver::Control(ctl) => ctl.release_external(symbol),
        }
    }
    pub fn statistics(&mut self) -> Result<StatisticsResult, ServerError> {
        match self {
            Solver::None => Err(ServerError::InternalError(
                "Solver::statistics failed! No control object.".to_string(),
            )),
            Solver::SolveHandle(_) => Err(ServerError::InternalError(
                "Solver::statistics failed! Solving has already started.".to_string(),
            )),
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
            Solver::None => Err(ServerError::InternalError(
                "Solver::configuration failed! No control object.".to_string(),
            )),
            Solver::SolveHandle(_) => Err(ServerError::InternalError(
                "Solver::configuration failed! Solving has already started.".to_string(),
            )),
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
            Solver::None => Err(ServerError::InternalError(
                "Solver::set_configuration failed! No control object.".to_string(),
            )),
            Solver::SolveHandle(_) => Err(ServerError::InternalError(
                "Solver::set_configuration failed! Solving has already started.".to_string(),
            )),
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
            Solver::None => Err(ServerError::InternalError(
                "Solver::solve_with_assumptions failed! No control object.".to_string(),
            )),
            Solver::SolveHandle(_) => Err(ServerError::InternalError(
                "Solver::solve_with_assumptions failed! Solving has already started.".to_string(),
            )),
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
                        return Err(ServerError::InternalError (
                            "Solver::solve_with_assumptions failed! \
                            The assumptions contain a literal that is not defined in the logic program.".to_string()
                        ));
                    }
                }
                self.solve(SolveMode::ASYNC | SolveMode::YIELD, &assumption_literals)
            }
        }
    }
    pub fn model(&mut self) -> Result<ModelResult, ServerError> {
        match self {
            Solver::None => Err(ServerError::InternalError(
                "Solver::model failed! No SolveHandle.".to_string(),
            )),
            Solver::Control(_) => Err(ServerError::InternalError(
                "Solver::model failed! Solving has not yet started.".to_string(),
            )),
            Solver::SolveHandle(SolveHandleWrapper::DLTheory(handle, dl_theory)) => {
                if handle.wait(Duration::ZERO) {
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
            Solver::SolveHandle(SolveHandleWrapper::ConTheory(handle, con_theory)) => {
                if handle.wait(Duration::ZERO) {
                    match handle.model_mut() {
                        Ok(Some(model)) => {
                            let mut buf = vec![];
                            write_model(model, &mut buf)?;
                            // TODO rewrite write_dl_theory_assignment to use boxed iterator
                            write_con_theory_assignment(
                                con_theory.borrow_mut().assignment(model.thread_id()?),
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
                if handle.wait(Duration::ZERO) {
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
            Solver::None => Err(ServerError::InternalError(
                "Solver::resume failed! No SolveHandle.".to_string(),
            )),
            Solver::Control(_) => Err(ServerError::InternalError(
                "Solver::resume failed! Solver has not yet started.".to_string(),
            )),
            Solver::SolveHandle(SolveHandleWrapper::DLTheory(handle, _)) => {
                handle.resume()?;
                Ok(())
            }
            Solver::SolveHandle(SolveHandleWrapper::ConTheory(handle, _)) => {
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
fn write_dl_theory_assignment<'a>(
    dl_theory_assignment: Box<dyn Iterator<Item = (Symbol, clingo::theory::TheoryValue)> + 'a>,
    mut out: impl io::Write,
) -> Result<(), io::Error> {
    for (symbol, theory_value) in dl_theory_assignment {
        writeln!(out, "{}={}", symbol, theory_value)?;
    }
    Ok(())
}
pub fn write_con_theory_assignment<'a>(
    con_theory_assignment: Box<dyn Iterator<Item = (Symbol, clingo::theory::TheoryValue)> + 'a>,
    mut out: impl io::Write,
) -> Result<(), io::Error> {
    for (symbol, theory_value) in con_theory_assignment {
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
#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r RequestId {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // The closure passed to `local_cache` will be executed at most once per
        // request: the first time the `RequestId` guard is used. If it is
        // requested again, `local_cache` will return the same value.
        request::Outcome::Success(
            request.local_cache(|| RequestId(ID_COUNTER.fetch_add(1, Ordering::Relaxed))),
        )
    }
}
pub struct Rewriter<'a> {
    builder: &'a mut ast::ProgramBuilder<'a>,
    theory: Rc<RefCell<DLTheory>>,
}
impl<'a> clingo::ast::StatementHandler for Rewriter<'a> {
    fn on_statement(&mut self, stm: &ast::Statement) -> bool {
        self.theory
            .borrow_mut()
            .rewrite_statement(stm, &mut self.builder)
    }
}
pub struct ConRewriter<'a> {
    builder: &'a mut ast::ProgramBuilder<'a>,
    theory: Rc<RefCell<ConTheory>>,
}
impl<'a> clingo::ast::StatementHandler for ConRewriter<'a> {
    fn on_statement(&mut self, stm: &ast::Statement) -> bool {
        self.theory
            .borrow_mut()
            .rewrite_statement(stm, &mut self.builder)
    }
}

/// recursively parse the statistics object
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

/// recursively parse the configuration object
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
