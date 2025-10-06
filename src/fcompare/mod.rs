use std::collections::HashMap;
use std::io::Write;

fn read_file(paths: &[impl AsRef<str>]) -> HashMap<String, String> {
    let mut kvs = Vec::new();

    for path in paths {
        let mut lines: Vec<_> = std::fs::read_to_string(path.as_ref())
            .unwrap()
            .split('\n')
            .filter(|x| !x.is_empty())
            .map(|x| {
                let (sum, path) = x.split_once("|").unwrap();
                (path.trim().to_string(), sum.trim().to_string())
            })
            .collect();

        kvs.append(&mut lines);
    }

    HashMap::from_iter(kvs)
}

pub fn main(left_dir: &str, right_dir: &str, partitions: &[&str], output_dir: &str) {
    let left_data = read_file(
        &partitions
            .iter()
            .map(|partition| format!("{left_dir}/{partition}.log"))
            .collect::<Box<[_]>>(),
    );

    let right_data = read_file(
        &partitions
            .iter()
            .map(|partition| format!("{right_dir}/{partition}.log"))
            .collect::<Box<[_]>>(),
    );

    let open_side = |side: &str| {
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(format!(
                "{output_dir}/not-in-{}.diff",
                side.split('/')
                    .filter(|x| !x.is_empty())
                    .next_back()
                    .unwrap()
            ))
            .unwrap()
    };

    let mut left_o = open_side(right_dir);
    let mut right_o = open_side(left_dir);

    let mut diff_o = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(format!("{output_dir}/diff.diff",))
        .unwrap();

    let mut left_keys = left_data
        .keys()
        .map(String::to_string)
        .collect::<Vec<String>>();
    left_keys.sort();

    let mut right_keys = right_data
        .keys()
        .map(String::to_string)
        .collect::<Vec<String>>();
    right_keys.sort();

    // if right key not in left keys
    for path in &right_keys {
        if left_data.contains_key(path) {
            continue;
        }

        writeln!(right_o, "{path}").unwrap();
    }

    for path in &left_keys {
        if right_data.contains_key(path) {
            continue;
        }

        writeln!(left_o, "{path}").unwrap();
    }

    for path in &left_keys {
        let Some(right_value) = right_data.get(path) else {
            continue;
        };

        if left_data.get(path).unwrap() == right_value {
            continue;
        }

        writeln!(diff_o, "{path}").unwrap();
    }
}
