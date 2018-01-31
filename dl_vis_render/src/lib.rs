extern crate dl_vis_layout;

pub fn render_file(toml: String) -> Result<String, String> {
    let new_line = "\n";
    match dl_vis_layout::layout_file(toml) {
        Ok((size, squares, lines)) => {
            let mut to_draw = String::new();
            for square in squares {
                let fig = format!("\\filldraw[fill=white, draw=black] ({0},{1}) rectangle ({2},{3});", square.left, square.upper, square.right, square.lower);
                to_draw = to_draw + &fig + new_line;
            }
            for line in lines {
                let fig = format!("\\draw ({0},{1}) -- ({2},{3});", line.x1, line.y1, line.x2, line.y2);
                to_draw = to_draw + &fig + new_line;
            }

            Ok(to_draw)
        },
        Err(err) => Err(err)
    }
}