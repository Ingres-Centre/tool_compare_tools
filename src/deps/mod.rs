use crate::deps::types::LibraryHolder;
use std::ops::Deref;

mod types;
mod utils;

pub(crate) fn main(ref_data_path: &str, src_data_path: &str, search: &[impl AsRef<str>]) {
    let ref_data = std::fs::read_to_string(ref_data_path).unwrap();
    let src_data = std::fs::read_to_string(src_data_path).unwrap();

    let existing_lib_paths: Box<[&str]> = src_data.split("\n").filter(|x| !x.is_empty()).collect();
    let existing_lib_names: Box<[&str]> = existing_lib_paths
        .iter()
        .map(|x| x.rsplit_once('/').unwrap().1)
        .collect();

    let holder = LibraryHolder::new(ref_data.deref());

    let mut found_libs = holder.search(search);
    found_libs.sort();

    for lib_ref in found_libs {
        let lib = lib_ref.borrow();

        if existing_lib_paths.contains(&lib.path) {
            println!("\x1b[90m{}\x1b[0m", lib.path);
        } else if existing_lib_names.contains(&lib.name) {
            println!(
                "\x1b[93m{:60}{} exists on SRC\x1b[0m",
                lib.path,
                existing_lib_paths
                    .get(
                        existing_lib_names
                            .iter()
                            .enumerate()
                            .find(|(_, d)| d == &&lib.name)
                            .unwrap()
                            .0
                    )
                    .unwrap()
            );
        } else {
            println!("\x1b[31m{}\x1b[0m", lib.path);
        };
    }
}
