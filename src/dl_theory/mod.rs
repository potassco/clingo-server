#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
mod bindings;

use clingo::theory::{Options, Theory, TheoryValue};
use clingo::{Control, Model, Statistics, Symbol};
use libloading::Library;
use libloading::Symbol as LibSymbol;
pub fn load_clingo_dl() {
    let lib = Library::new("./libclingo-dl.so").unwrap();
    unsafe {
        let create: LibSymbol<unsafe extern "C" fn(name: *const *const clingodl_theory) -> bool> =
            lib.get(b"clingodl_create").unwrap();
        let theory: *const clingodl_theory_t = std::ptr::null();
        let what = create(&theory);
        println!("What:{}", what);
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clingodl_theory {
    _unused: [u8; 0],
}
pub type clingodl_theory_t = clingodl_theory;
pub struct DLTheory {}
impl Theory for DLTheory {
    /// creates the theory
    fn create() -> Self {
        DLTheory {}
    }
    /// registers the theory with the control
    fn register(&mut self, ctl: &mut Control) -> bool {
        true
    }
    /// prepare the theory between grounding and solving
    fn prepare(&mut self, ctl: &mut Control) -> bool {
        true
    }
    /// add options for your theory
    fn register_options(&mut self, options: &mut Options) -> bool {
        true
    }
    /// validate options for your theory
    fn validate_options(&mut self) -> bool {
        true
    }
    /// callback on every model
    fn on_model(&mut self, model: &mut Model) -> bool {
        true
    }
    /// callback on statistic updates
    /// please add a subkey with the name of your theory
    fn on_statistics(&mut self, step: &mut Statistics, akku: &mut Statistics) -> bool {
        true
    }
    /// obtain a symbol index which can be used to get the value of a symbol
    /// returns true if the symbol exists
    /// does not throw
    fn lookup_symbol(&mut self, symbol: Symbol, index: &mut usize) -> bool {
        true
    }
    /// obtain the symbol at the given index
    /// does not throw
    fn get_symbol(&mut self, index: usize) -> clingo::Symbol {
        clingo::Symbol::create_id("test", true).unwrap()
    }
    /// initialize index so that it can be used with clingodl_assignment_next
    /// does not throw
    fn assignment_begin(&mut self, thread_id: u32, index: &mut usize) {}
    /// move to the next index that has a value
    /// returns true if the updated index is valid
    /// does not throw
    fn assignment_next(&mut self, thread_id: u32, index: &mut usize) -> bool {
        true
    }
    /// check if the symbol at the given index has a value
    /// does not throw
    fn assignment_has_value(&mut self, thread_id: u32, index: usize) -> bool {
        true
    }
    /// get the symbol and it's value at the given index
    /// does not throw
    fn assignment_get_value(&mut self, thread_id: u32, index: usize, value: &mut TheoryValue) {}
    /// configure theory manually (without using clingo's options facility)
    /// Note that the theory has to be configured before registering it and cannot be reconfigured.
    fn configure(&mut self, key: &str, value: &str) -> bool {
        true
    }
}
