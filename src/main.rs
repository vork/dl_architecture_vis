extern crate dl_vis_render as render;

use std::fs::File;
use std::io::prelude::*;

fn main() {
    println!("Hello, world!");

    let toml_str = r#"
start = 1
start_align_left = true
start_align_up = true
end = 4

[[nodes]]
	id = 1
	dimension = [5, 512, 512, 1]
	pass_to = 2
	above_of = 2

[[nodes]]
	id = 2
	dimension = [5, 512, 512, 1]
	left_of = 3
	above_of = 3
	pass_to = 3

[[nodes]]
	id = 3
	dimension = [5, 256, 256, 1]
	left_of = 4
	below_of = 4
    skip_connection_to = 4

[[nodes]]
	id = 4
	dimension = [5, 256, 256, 1]
    "#;

    println!("{}", render::render_file(toml_str.to_string()).unwrap());

    //let mut file = File::create("image.svg").unwrap();
    //file.write_all(&(render::render_file(toml_str.to_string()).unwrap()).as_bytes());
}
