fn visit_dir(libs: &mut Vec<String>, path: &str) {
    let it = std::fs::read_dir(path).unwrap();

    for entry in it.flatten() {
        if entry.file_type().unwrap().is_dir() {
            visit_dir(libs, entry.path().to_str().unwrap());
            continue;
        }

        if entry.file_type().unwrap().is_file() && entry.path().ends_with("proprietary-files.txt") {
            let mut new_libs: Vec<String> = std::fs::read_to_string(entry.path())
                .unwrap()
                .split('\n')
                .filter_map(|x| {
                    if x.is_empty() || x.starts_with("#") || !x.contains('/') {
                        return None;
                    }

                    Some(format!(
                        "/{}",
                        x.split_once(';').map(|(path, _)| path).unwrap_or(x)
                    ))
                })
                .collect();

            libs.append(&mut new_libs);
        }
    }
}

pub fn compile(base_list_path: impl AsRef<str>, device_dir_path: impl AsRef<str>) -> Box<[String]> {
    let mut libs: Vec<String> = std::fs::read_to_string(base_list_path.as_ref())
        .unwrap()
        .split('\n')
        .filter(|x| !x.is_empty())
        .map(str::to_string)
        .collect();

    visit_dir(&mut libs, device_dir_path.as_ref());
    libs.into_boxed_slice()
}
