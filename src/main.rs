use nalgebra::Vector3;
use obj::Obj;
use obj_to_pathfinding_grid::geometry::Triangle;
use obj_to_pathfinding_grid::{convert, parse_triangles, Preprocessor, Progress};
use std::fs;
use std::fs::create_dir_all;
use std::path::PathBuf;
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
        let input_path = &entry.path();

        match input_path.extension() {
            Some(v) if v != "obj" => continue,
            None => continue,
            _ => {}
        }

        let input_name = input_path
            .file_name()
            .map(|f| f.to_str())
            .flatten()
            .to_owned()
            .expect("Failed to get file name");

        let map_id = &input_name[3..7];
        let map_output_name = format!("map_{}", map_id);

        let y = input_name[7..9]
            .parse::<u32>()
            .expect("Failed to parse tile y from file name");

        let x = input_name[9..11]
            .parse::<u32>()
            .expect("Failed to parse tile x from file name");

        let map_output_path = output.join(map_output_name);

        if !map_output_path.exists() {
            create_dir_all(&map_output_path).expect("Failed to create output folder");
        }

        let output_name = format!("tile_{}_{}.{}", x, y, "dat");
        let output_path = map_output_path.join(&output_name);

        let obj = Obj::load(input_path).expect("Failed to load obj file");

        let center_x = tile_to_axis(x) - 533.3 * 0.5;
        let center_y = tile_to_axis(y) - 533.3 * 0.5;

        let center = Vector3::new(center_x, center_y, 0.0);

        let progress = StdOutProgress::new();
        let preprocessor = WowPreprocessor::new(x, y);

        println!(
            "Converting obj file {} to grid {} with center x: {}, y: {}",
            input_name, &output_name, center_x, center_y
        );

        let triangles: Vec<Triangle> = parse_triangles(&obj).into_iter().collect();

        convert(triangles, center, 534, 1000, progress, preprocessor)
            .export(output_path)
            .expect("Failed to save output file");
    }
}

struct StdOutProgress {}

impl StdOutProgress {
    pub fn new() -> Self {
        StdOutProgress {}
    }
}

impl Progress for StdOutProgress {
    fn update_progress(&self, percent: f32) {
        print!("Current progress: {:.2}%\r", percent)
    }
}

struct WowPreprocessor {
    tile_x: u32,
    tile_z: u32,
}

impl WowPreprocessor {
    pub fn new(tile_x: u32, tile_z: u32) -> Self {
        WowPreprocessor { tile_x, tile_z }
    }

    pub fn in_same_tile(&self, vector: &Vector3<f32>) -> bool {
        let tile_x = axis_to_tile(vector.x);
        let tile_z = axis_to_tile(vector.z);

        self.tile_x == tile_x && self.tile_z == tile_z
    }
}

impl Preprocessor for WowPreprocessor {
    fn pre_process(
        &self,
        triangle: Triangle,
        _width: u32,
        _height: u32,
        _center: Vector3<f32>,
    ) -> Option<Triangle> {
        let a_result = self.in_same_tile(&triangle.a);
        let b_result = self.in_same_tile(&triangle.b);
        let c_result = self.in_same_tile(&triangle.c);

        // Triangle completely out of tile. Skipping it.
        if !a_result && !b_result && !c_result {
            return None;
        }

        Some(swap_triangle_y_z(triangle))
    }
}

// Y is height in our data.
fn swap_triangle_y_z(triangle: Triangle) -> Triangle {
    let a = swap_vector_y_z(triangle.a);
    let b = swap_vector_y_z(triangle.b);
    let c = swap_vector_y_z(triangle.c);

    Triangle::new(a, b, c)
}

fn swap_vector_y_z(vector: Vector3<f32>) -> Vector3<f32> {
    Vector3::new(vector.x, vector.z, vector.y)
}

fn axis_to_tile(axis: f32) -> u32 {
    (32.0 - (axis / 533.33)).floor() as u32
}

fn tile_to_axis(tile: u32) -> f32 {
    (32.0 - tile as f32) * 533.3
}

#[cfg(test)]
mod tests {
    use crate::{axis_to_tile, tile_to_axis};

    #[test]
    fn test_axis_to_tile() {
        let tile_x = axis_to_tile(375.0);
        let tile_z = axis_to_tile(-13904.166016);

        assert_eq!(tile_x, 31);
        assert_eq!(tile_z, 58);
    }

    #[test]
    fn test_tile_to_axis() {
        let x = tile_to_axis(31);
        let z = tile_to_axis(58);

        assert_eq!(x, 533.3);
        assert_eq!(z, -13865.8);
    }
}
