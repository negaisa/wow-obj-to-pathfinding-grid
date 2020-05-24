use obj::Obj;
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "wow-obj-to-pathfinding-grid")]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    input_folder: PathBuf,
    #[structopt(short, long, default_value = "grid", parse(from_os_str))]
    output_folder: PathBuf,
}

fn main() {
    let opt: Opt = Opt::from_args();

    let input = &opt.input_folder;
    let output = &opt.output_folder;

    let files_in_dir = fs::read_dir(input).expect("Failed to read directory");

    for result_entry in files_in_dir {
        let entry = result_entry.expect("Failed to read file in directory");
        let path = entry.path();

        match path.extension() {
            Some(v) if v != "obj" => continue,
            None => continue,
            _ => {}
        }

        let obj = Obj::load(path).expect("Failed to load obj file");
    }
}
