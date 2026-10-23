// Gives the Linux library a SONAME, so programs linked against it look for
// `libconsole_launcher_engine.so` by name (next to them, through RPATH)
// instead of the path it had at build time.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,libconsole_launcher_engine.so");
    }
}
