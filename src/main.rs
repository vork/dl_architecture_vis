extern crate dl_vis_layout as layout;

fn main() {
    println!("Hello, world!");

    let toml_str = r#"
start = 1
start_align_left = true
start_align_up = true
end = 3

[[nodes]]
	id = 1
	dimension = [5, 512, 512, 1]
	pass_to = 2
	left_of = 2

[[nodes]]
	id = 2
	dimension = [5, 512, 512, 1]
	above_of = 3
	[nodes.operation]
		to = 3
		[nodes.operation.convolution]
			dimension = 3
			kernel_size = 3
			num_outputs = 128
			stride = [1, 2, 2]
			activation_fn = "relu"

[[nodes]]
	id = 3
	dimension = [5, 256, 256, 1]
    "#;

    layout::render_file(toml_str.to_string());
}
