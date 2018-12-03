
extern crate iron;
extern crate staticfile;
extern crate mount;

// This example serves the docs from target/doc/staticfile at /doc/
//
// Run `cargo doc && cargo run --example doc_server`, then
// point your browser to http://127.0.0.1:3000/doc/

use std::path::Path;

use iron::Iron;
use staticfile::Static;
use mount::Mount;

fn main() {
    //! Main.
    let mut mount = Mount::new();

    mount.mount("/", Static::new(Path::new("/release")));
    Iron::new(mount).http("0.0.0.0:8000").unwrap();
}