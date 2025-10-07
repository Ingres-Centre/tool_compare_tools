use crate::deps::types::LibraryRef;
use std::rc::Rc;

pub fn search_library<'a>(
    libs: &[LibraryRef<'a>],
    name_or_path: &str,
    is_32_bit: Option<bool>,
) -> Option<LibraryRef<'a>> {
    let search_is_32_bit =
        is_32_bit.unwrap_or_else(|| name_or_path.rsplit_once('/').unwrap().0.contains("lib64"));

    if name_or_path.contains('/') {
        if let Some(lib) = libs.iter().find(|lib_ref| {
            let Ok(lib) = lib_ref.try_borrow() else {
                return false;
            };

            lib.path == name_or_path
        }) {
            return Some(lib.clone());
        }

        let search_name = name_or_path.rsplit_once('/').unwrap().1;

        libs.iter().find(|lib_ref| {
            let Ok(lib) = lib_ref.try_borrow() else {
                return false;
            };

            lib.is_32_bit == search_is_32_bit && lib.name == search_name
        })
    } else {
        if let Some(lib) = libs.iter().find(|lib_ref| {
            let Ok(lib) = lib_ref.try_borrow() else {
                return false;
            };

            !lib.is_32_bit && lib.name == name_or_path
        }) {
            return Some(lib.clone());
        }

        libs.iter().find(|lib_ref| {
            let Ok(lib) = lib_ref.try_borrow() else {
                return false;
            };

            lib.is_32_bit && lib.name == name_or_path
        })
    }
    .map(Rc::clone)
}
