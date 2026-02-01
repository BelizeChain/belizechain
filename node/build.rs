//! Build script for BelizeChain node
//! Sets implementation version for Substrate CLI integration

fn main() {
    // Set implementation version for substrate CLI
    if std::env::var("SUBSTRATE_CLI_IMPL_VERSION").is_err() {
        let version = env!("CARGO_PKG_VERSION");
        println!("cargo:rustc-env=SUBSTRATE_CLI_IMPL_VERSION={}", version);
    }
    
    // Tell cargo to rerun this build script if the version changes
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
}
