use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=links.c");
    println!("cargo:rustc-link-lib=static=links");

    // Produces headers without any definitions passed:
    let bindings = bindgen::Builder::default()
        .header("src/links.c")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

     // Write bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Produces binary with some macros defined:
    cc::Build::new()
        .define("STBI_NO_STDIO", Some(""))
        .define("STBI_WRITE_NO_STDIO", Some(""))
        .define("STB_IMAGE_IMPLEMENTATION", Some(""))
        .define("STB_IMAGE_RESIZE_IMPLEMENTATION", Some(""))
        .define("STB_IMAGE_WRITE_IMPLEMENTATION", Some(""))
        .define("MY_IMPL", Some(""))
        .include("/usr/local/include/stb")
        .file("src/links.c")
        .compile("links");
}
