// option3.rs
// Make me compile! Execute `rustlings hint option3` for hints

struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let y: Option<Point> = Some(Point { x: 100, y: 200 });

    // Notes to self:
    // Apparently, when accessing the `Some` value of an `Option`, the value will get moved out.
    // The variable `p` took the ownership of the `Point` value when accessing the `Some` of `y`, so `y` can't be used anymore
    // unless we pass `y` by ref then `p` itself is also a ref.
    match &y {
        Some(p) => println!("Co-ordinates are {},{} ", p.x, p.y),
        _ => println!("no match"),
    }
    y; // Fix without deleting this line.
}
