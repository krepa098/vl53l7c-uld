use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let uld_root = format!("{manifest_dir}/vendor/VL53L7CX_ULD_driver_2.0.0");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // compile the uld as static library
    let mut cc = cc::Build::new();
    cc.file(format!("{uld_root}/VL53L7CX_ULD_API/src/vl53l7cx_api.c"))
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
        .out_dir(out_path.to_str().unwrap());

    // features
    // use this to reduce the RAM size and I2C size
    #[allow(unused_mut)]
    let mut defines: Vec<&str> = vec![];

    #[cfg(feature = "disable_ambient_per_spad")]
    defines.push("VL53L7CX_DISABLE_AMBIENT_PER_SPAD");
    #[cfg(feature = "disable_nb_spads_enable")]
    defines.push("VL53L7CX_DISABLE_NB_SPADS_ENABLED");
    #[cfg(feature = "disable_nb_target_detected")]
    defines.push("VL53L7CX_DISABLE_NB_TARGET_DETECTED");
    #[cfg(feature = "disable_signal_per_spad")]
    defines.push("VL53L7CX_DISABLE_SIGNAL_PER_SPAD");
    #[cfg(feature = "disable_range_sigma_mm")]
    defines.push("VL53L7CX_DISABLE_RANGE_SIGMA_MM");
    #[cfg(feature = "disable_target_status")]
    defines.push("VL53L7CX_DISABLE_TARGET_STATUS");
    #[cfg(feature = "disable_motion_indicator")]
    defines.push("VL53L7CX_DISABLE_MOTION_INDICATOR");
    #[cfg(feature = "disable_distance_mm")]
    defines.push("VL53L7CX_DISABLE_DISTANCE_MM");
    #[cfg(feature = "disable_reflectance_percent")]
    defines.push("VL53L7CX_DISABLE_REFLECTANCE_PERCENT");
    #[cfg(feature = "plugin_xtalk")]
    defines.push("PLUGIN_XTALK");

    for def in &defines {
        cc.define(def, None);
    }

    // compile as static lib
    cc.compile("VL53L7CX_ULD");

    // link to the static uld lib
    println!("cargo:rustc-link-search={}", out_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=VL53L7CX_ULD");

    let mut bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{uld_root}/VL53L7CX_ULD_API/inc"))
        .clang_arg(format!("-I{uld_root}/Platform"))
        .use_core()
        .fit_macro_constants(true)
        .clang_macro_fallback()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    for def in &defines {
        bindings = bindings.clang_arg(format!("-D{}", def));
    }

    // find certain header files ('string.h', 'stddef.h') not picked up automatically when cross compiling
    let target = std::env::var("TARGET").unwrap();
    if target == "thumbv7em-none-eabihf" {
        // find the gcc-arm-none-eabi version
        let mut gcc_versions =
            std::fs::read_dir("/usr/lib/gcc/arm-none-eabi").expect("Cannot find gcc version");
        let gcc_version = gcc_versions
            .next()
            .unwrap()
            .unwrap()
            .file_name()
            .to_str()
            .unwrap()
            .to_owned();
        println!("gcc version: '{}'", gcc_version);

        // pass newlib and gcc arm-none-eabi include
        bindings = bindings
            .clang_arg("-I/usr/include/newlib")
            .clang_arg(format!(
                "-I/usr/lib/gcc/arm-none-eabi/{}/include",
                gcc_version
            ));
    }

    let bindings = bindings.generate().expect("Unable to generate bindings");

    // write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
