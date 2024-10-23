use sdl2::{pixels::Color, rect::Point, render::Canvas, video::Window};


fn draw_line(canvas: &mut Canvas<Window>, p1: Point, p2: Point, color: Color) {
    let _ = canvas.set_draw_color(color);
    let _ = canvas.draw_line(p1, p2);
}

fn fill_flat_bottom_triangle(canvas: &mut Canvas<Window>, p1: Point, p2: Point, p3: Point, color: Color) {
    let inv_slope1 = (p2.x - p1.x) as f32 / (p2.y - p1.y) as f32;
    let inv_slope2 = (p3.x - p1.x) as f32 / (p3.y - p1.y) as f32;

    let mut curx1 = p1.x as f32;
    let mut curx2 = p1.x as f32;

    for y in p1.y..=p2.y {
        let _ = canvas.set_draw_color(color);
        let _ = canvas.draw_line(Point::new(curx1 as i32, y), Point::new(curx2 as i32, y));
        curx1 += inv_slope1;
        curx2 += inv_slope2;
    }
}

fn fill_flat_top_triangle(canvas: &mut Canvas<Window>, p1: Point, p2: Point, p3: Point, color: Color) {
    let inv_slope1 = (p3.x - p1.x) as f32 / (p3.y - p1.y) as f32;
    let inv_slope2 = (p3.x - p2.x) as f32 / (p3.y - p2.y) as f32;

    let mut curx1 = p3.x as f32;
    let mut curx2 = p3.x as f32;

    for y in (p1.y..=p3.y).rev() {
        let _ = canvas.set_draw_color(color);
        let _ = canvas.draw_line(Point::new(curx1 as i32, y), Point::new(curx2 as i32, y));
        curx1 -= inv_slope1;
        curx2 -= inv_slope2;
    }
}

pub fn draw_filled_triangle(canvas: &mut Canvas<Window>, mut p1: Point, mut p2: Point, mut p3: Point, color: Color) {
    // Sort points by y-coordinate (ascending order)
    if p2.y < p1.y {
        std::mem::swap(&mut p1, &mut p2);
    }
    if p3.y < p1.y {
        std::mem::swap(&mut p1, &mut p3);
    }
    if p3.y < p2.y {
        std::mem::swap(&mut p2, &mut p3);
    }

    if p2.y == p3.y {
        // Flat-bottom triangle
        fill_flat_bottom_triangle(canvas, p1, p2, p3, color);
    } else if p1.y == p2.y {
        // Flat-top triangle
        fill_flat_top_triangle(canvas, p1, p2, p3, color);
    } else {
        // General triangle (split into a flat-bottom and flat-top triangle)
        let new_x = p1.x + ((p2.y - p1.y) * (p3.x - p1.x)) / (p3.y - p1.y);
        let new_point = Point::new(new_x, p2.y);
        fill_flat_bottom_triangle(canvas, p1, p2, new_point, color);
        fill_flat_top_triangle(canvas, p2, new_point, p3, color);
    }
}
