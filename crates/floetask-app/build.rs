//! Embeds the floetask icon into the Windows executable.

fn main() {
    println!("cargo:rerun-if-changed=../../assets/floetask.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("../../assets/floetask.ico");
        if let Err(error) = resource.compile() {
            // A missing resource compiler only costs the exe icon.
            println!("cargo:warning=could not embed the exe icon: {error}");
        }
    }
}
