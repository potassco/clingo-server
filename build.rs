use bindgen::builder;
// use std::env;
// use std::path::PathBuf;
// use std::process::Command;
fn main() {

    // let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR was not set"));
    // let clingo_dl_dir = out_dir.join("clingo-dl");

    // if !clingo_dl_dir.exists() {
    //     Command::new("git")
    //         .args(&["clone", "https://github.com/potassco/clingo-dl.git"])
    //         .current_dir(out_dir.to_str().unwrap())
    //         .status()
    //         .unwrap();

    //     Command::new("git")
    //         .args(&["checkout", "wip"])
    //         .current_dir(clingo_dl_dir.to_str().unwrap())
    //         .status()
    //         .unwrap();

    //     Command::new("git")
    //         .args(&["submodule", "update", "--init", "--recursive"])
    //         .current_dir(clingo_dl_dir.to_str().unwrap())
    //         .status()
    //         .unwrap();
    // }
    // Configure and generate bindings.
    let bindings = builder()
        .header("./clingo-dl.h")
        // TODO: use clingo-dl.h from the clingo-dl repo
        // .header(clingo_dl_dir.join("libclingo-dl/clingo-dl.h").to_str().unwrap())
        .whitelist_type("clingodl_theory_t")
        .whitelist_function("clingodl_create")
        .whitelist_function("clingodl_destroy")
        .whitelist_function("clingodl_register")
        .whitelist_function("clingodl_rewrite_statement")
        .whitelist_function("clingodl_prepare")
        .whitelist_function("clingodl_register_options")
        .whitelist_function("clingodl_validate_options")
        .whitelist_function("clingodl_on_model")
        .whitelist_function("clingodl_on_statistics")
        .whitelist_function("clingodl_lookup_symbol")
        .whitelist_function("clingodl_get_symbol")
        .whitelist_function("clingodl_assignment_begin")
        .whitelist_function("clingodl_assignment_next")
        .whitelist_function("clingodl_assignment_has_value")
        .whitelist_function("clingodl_assignment_get_value")
        .whitelist_function("clingodl_configure")
        .size_t_is_usize(true)
        .generate()
        .unwrap();

    // Write the generated bindings to an output file.
    bindings.write_to_file("src/dl_theory/bindings.rs").unwrap();
}
