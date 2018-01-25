extern crate simplesvg;
extern crate dl_vis_layout;

use simplesvg::{Attr, Color, Fig, Svg};

pub fn render_file(toml: String) -> Result<String, String> {
    match dl_vis_layout::layout_file(toml) {
        Ok((size, squares, lines)) => {
            let mut to_draw = Vec::new();
            for square in squares {
                let fig = Fig::Rect(square.left, square.upper, square.right - square.left, square.lower - square.upper)
                    .styled(Attr::default().fill(Color(0xff, 0, 0)));
                to_draw.push(fig);
            }
            for line in lines {
                let fig = Fig::Line(line.x1, line.y1, line.x2, line.y2)
                    .styled(Attr::default().stroke(Color(0, 0, 0)).stroke_width(1.));
                to_draw.push(fig);
            }

            Ok(Svg(to_draw, size.0 as u32, size.1 as u32).to_string())
        },
        Err(err) => Err(err)
    }
}