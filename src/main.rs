mod deps;
mod fcompare;
mod libs_list;

fn main() {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();

    let (bin, args) = args.split_first().unwrap();

    let (action, args) = args
        .split_first()
        .map(|(x, r)| (x.to_string(), r))
        .unwrap_or((String::new(), args));

    match action.as_str() {
        "fcmp" => {
            if args.len() < 4 {
                log::warn!("TODO: Add more documentation.");
                log::warn!("Usage:");
                log::warn!(
                    "{bin} {action} <left dir> <right dir> <partitions divided by slash> <output dir>",
                );
            }

            let (left_dir, args) = args.split_first().unwrap();
            let (right_dir, args) = args.split_first().unwrap();
            let (partitions, args) = args.split_first().unwrap();
            let (output_dir, _) = args.split_first().unwrap();

            let partitions = partitions.split("/").collect::<Vec<&str>>();

            fcompare::main(left_dir, right_dir, &partitions, output_dir);
        }
        "deps" => {
            if args.len() < 3 {
                log::warn!("Usage:");
                log::warn!(
                    "{bin} {action} <path to list of libs and deps from REF> <path to list of existing libs from SRC> [lib name or absolute path]*",
                );
                return;
            }

            let (ref_data_path, rem) = args.split_first().unwrap();
            let (src_data_path, search) = rem.split_first().unwrap();

            deps::main(ref_data_path, src_data_path, search);
        }
        "lib-list" => {
            if args.len() < 3 {
                log::warn!("Usage:");
                log::warn!(
                    "{bin} {action} <path to base libraries list> <path to device directory in AOSP workspace> <path to output file>",
                );
                return;
            }
            let (base_list_path, args) = args.split_first().unwrap();
            let (device_dir_path, args) = args.split_first().unwrap();
            let (output_file, _) = args.split_first().unwrap();

            let list = libs_list::compile(base_list_path, device_dir_path).join("\n");

            std::fs::write(output_file, list).unwrap();
        }
        var => {
            log::warn!("Unknown action {}!", var);
        }
    }
}
