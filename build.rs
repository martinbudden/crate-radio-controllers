fn main() {
    // Tell Cargo that 'rp' is an expected custom cfg flag name so it doesn't warn us
    println!("cargo::rustc-check-cfg=cfg(rp)");
    // println!("cargo::rustc-check-cfg=cfg(nightly)");

    // ----------------------------------------------------
    // RP Target Chip Selection & CFG Generation
    // ----------------------------------------------------

    // Gather all active RP chip features from Cargo's environment variables
    let has_rp2040 = std::env::var_os("CARGO_FEATURE_RP2040").is_some();
    let has_rp235xa = std::env::var_os("CARGO_FEATURE_RP235XA").is_some();
    let has_rp235xb = std::env::var_os("CARGO_FEATURE_RP235XB").is_some();

    // Count how many chips are selected
    let count = [has_rp2040, has_rp235xa, has_rp235xb].iter().filter(|&&active| active).count();

    // Safety Check: Prevent multiple mutually exclusive chips from building
    if count > 1 {
        panic!(
            "\n\nError: Multiple RP chip features were enabled simultaneously!\n\
             Please select exactly one target chip feature.\n"
        );
    }

    // Emit the unified 'rp' config flag if exactly one chip is chosen
    if count == 1 {
        println!("cargo:rustc-cfg=rp");
    }
}
