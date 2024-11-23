use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let uld_root = format!("{manifest_dir}/vendor/VL53L7CX_ULD_driver_2.0.0");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // compile the uld as static library
    cc::Build::new()
        .file(format!("{uld_root}/VL53L7CX_ULD_API/src/vl53l7cx_api.c"))
        .file(format!(
            "{uld_root}/VL53L7CX_ULD_API/src/vl53l7cx_plugin_detection_thresholds.c"
        ))
        .file(format!(
            "{uld_root}/VL53L7CX_ULD_API/src/vl53l7cx_plugin_motion_indicator.c"
        ))
        .file(format!(
            "{uld_root}/VL53L7CX_ULD_API/src/vl53l7cx_plugin_xtalk.c"
        ))
        .include(format!("{uld_root}/VL53L7CX_ULD_API/inc"))
        .include(format!("{uld_root}/Platform"))
        .static_flag(true)
        .out_dir(out_path.to_str().unwrap())
        .compile("VL53L7CX_ULD");

    // link to the static uld lib
    println!("cargo:rustc-link-search={}", out_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=VL53L7CX_ULD");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{uld_root}/VL53L7CX_ULD_API/inc"))
        .clang_arg(format!("-I{uld_root}/Platform"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
