extern crate toml;

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path;

fn main() {
    let mut lock_buf = String::new();
    fs::File::open("Cargo.lock")
        .unwrap()
        .read_to_string(&mut lock_buf)
        .unwrap();
    let lock_toml = toml::Parser::new(&lock_buf).parse().unwrap();

    let mut packages = Vec::new();
    for package in lock_toml.get("package").unwrap().as_slice().unwrap() {
        let package = package.as_table().unwrap();
        packages.push((
            package.get("name").unwrap().as_str().unwrap(),
            package.get("version").unwrap().as_str().unwrap(),
        ));
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let mut versions_file =
        fs::File::create(&path::Path::new(&out_dir).join("versions.include")).unwrap();
    versions_file
        .write(
            format!(
                "pub const MP_VERSION: &'static str = \"{}\";",
                packages
                    .iter()
                    .filter(|p| p.0 == "maker-panel")
                    .map(|p| p.1)
                    .collect::<Vec<_>>()
                    .join("")
            )
            .as_ref(),
        )
        .unwrap();
}
