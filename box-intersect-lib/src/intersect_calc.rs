use crate::box_def::Box;
use std::cmp::{max, min};

pub fn intersect_area(b1: &Box, b2: &Box) -> u64 {
    let lx1 = max(b1.x1, b2.x1);
    let lx2 = min(b1.x2(), b2.x2());
    let ly1 = max(b1.y1, b2.y1);
    let ly2 = min(b1.y2(), b2.y2());
    let lw = max(lx2 - lx1, 0);
    let lh = max(ly2 - ly1, 0);
    lw as u64 * lh as u64
}

pub fn does_intersect(b1: &Box, b2: &Box) -> bool {
    b1.y2() > b2.y1 && b1.y1 < b2.y2() && b1.x2() > b2.x1 && b1.x1 < b2.x2()
}

pub fn union_area(b1: &Box, b2: &Box) -> u64 {
    b1.area() + b2.area() - intersect_area(b1, b2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_boxes_area() {
        let mut rng = rand::thread_rng();
        let region_size = 400;
        let num_boxes = 1000;
        let max_box_size = 100;
        for _ in 0..num_boxes {
            let b1 = Box {
                x1: rng.gen_range(0..region_size - max_box_size),
                y1: rng.gen_range(0..region_size - max_box_size),
                xs: rng.gen_range(1..max_box_size as u32),
                ys: rng.gen_range(1..max_box_size as u32),
            };
            let b2 = Box {
                x1: rng.gen_range(0..region_size - max_box_size),
                y1: rng.gen_range(0..region_size - max_box_size),
                xs: rng.gen_range(1..max_box_size as u32),
                ys: rng.gen_range(1..max_box_size as u32),
            };
            assert_eq!(does_intersect(&b1, &b2), (intersect_area(&b1, &b2) != 0));
        }
    }
}
