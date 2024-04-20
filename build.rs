extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cpp/clipper.cpp");
    println!("cargo:rerun-if-changed=cpp/clipper.hpp");
    println!("cargo:rerun-if-changed=cpp/clipper_wrap.cpp");
    println!("cargo:rerun-if-changed=cpp/clipper_wrap.h");

    let mut config = cc::Build::new();
    config.cpp(true);
    config.flag("-std=c++0x");
    config.flag("-Wno-deprecated-copy");
    config.flag("-Wno-class-memaccess");
    config.flag("-Wno-unused-parameter");
    config.file("cpp/clipper.cpp");
    config.file("cpp/clipper_wrap.cpp");
    config.include("cpp");
    config.compile("clipper.a");

    let bindings = bindgen::Builder::default()
        .header("cpp/clipper_wrap.h")
        .clang_arg("-xc++")
        .layout_tests(false)
        .prepend_enum_name(false)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
