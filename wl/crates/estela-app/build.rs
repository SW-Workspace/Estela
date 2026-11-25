fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg-bin=estela-app=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg-bin=estela-app=/ENTRY:mainCRTStartup");
    } else {
        println!("cargo:rustc-link-arg-bin=estela-app=-mwindows");
    }
}
