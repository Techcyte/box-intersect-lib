use std::cmp::{max, min};

use crate::box_def::Box;

#[derive(Clone, Copy)]
struct CalcBox {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

#[derive(Copy, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

fn does_include(outer: &CalcBox, inner: &CalcBox) -> bool {
    inner.x1 >= outer.x1 && inner.y1 >= outer.y1 && outer.x2 >= inner.x2 && outer.y2 >= inner.y2
}

fn get_input_errors(all_boxes: &[Box], tile_width: u32, tile_height: u32) -> Result<(), String> {
    for b in all_boxes.iter() {
        if !(b.xs <= tile_width && b.ys <= tile_height) {
            return Err(format!(
                "Box has size: ({}, {}), could not fit in tile of size: ({}, {})",
                b.xs, b.ys, tile_width, tile_height
            ));
        }
    }
    Ok(())
}

pub fn efficient_coverage(
    all_boxes: &[Box],
    tile_width: u32,
    tile_height: u32,
) -> Result<Vec<(Point, Vec<u32>)>, String> {
    /* Generates an approximately minimal set of tiles that completely covers all the boxes.

    Currently uses a left-to-right line sweep algorithm with visualizations in the docs.
    While the implementation here is slightly different than shown in the docs, its outputs
    are exactly equivalent to the documented visualizations, it just performs the operations
    in a different order.

    Returned points represent the top left corners of the resulting tiles, and the box
    indicies that go within the tile.

    Returns an error if any of the boxes are too big and cannot be covered by a single tile
    (i.e. are larger than the tile) */

    // Check input validity
    get_input_errors(all_boxes, tile_width, tile_height)?;

    // argsort boxes by x axis so we can sweep tile canidates from right to left
    let mut sorted_by_x: Vec<_> = all_boxes
        .iter()
        .enumerate()
        .map(|(bidx, b)| (*b, bidx as u32))
        .collect();
    // use y as a tie-breaker
    sorted_by_x.sort_unstable_by_key(|v| (v.0.x1, v.0.y1));

    let mut current_region_list: Vec<CalcBox> = Vec::new();
    let mut final_assignment: Vec<Vec<u32>> = Vec::new();
    let mut start_idx = 0;

    for (b1b, b1idx) in sorted_by_x.iter() {
        let b1 = CalcBox {
            x1: b1b.x1,
            y1: b1b.y1,
            x2: b1b.x2(),
            y2: b1b.y2(),
        };
        // remove all tile regions which couldn't possibly include this box, or any future boxes
        start_idx += current_region_list[start_idx..]
            .iter()
            .take_while(|region| region.x2 < b1.x1)
            .count();
        // this is the largest region of possible tiles which includes b1 as its left-most box
        let b1_region = CalcBox {
            x1: b1.x1,
            y1: b1.y2 - tile_height as i32,
            x2: b1.x1 + tile_width as i32,
            y2: b1.y1 + tile_height as i32,
        };
        let b1_idx = current_region_list.len();
        // find the tile region that could include this box with the lowest x1 value
        // if it does not exist, then use the new region
        let (best_idx_offset, best_region) = current_region_list[start_idx..]
            .iter()
            .copied()
            .enumerate()
            .find(|(_new_idx, new_region)| does_include(new_region, &b1))
            .unwrap_or((b1_idx - start_idx, b1_region));

        let best_idx = best_idx_offset + start_idx;

        if best_idx == b1_idx {
            final_assignment.push(vec![*b1idx]);
            current_region_list.push(best_region);
        } else {
            // shrink the tile region to make sure that all possible tiles in the region
            // will include the current box b1
            let orig_min_y = best_region.y2 - tile_height as i32;
            let orig_max_y = best_region.y1 + tile_height as i32;

            let min_y = min(orig_min_y, b1.y1);
            let max_y = max(orig_max_y, b1.y2);

            let new_y1 = max_y - tile_height as i32;
            let new_y2 = min_y + tile_height as i32;

            let new_canidate_region = CalcBox {
                x1: best_region.x1,
                x2: best_region.x2,
                y1: new_y1,
                y2: new_y2,
            };
            final_assignment[best_idx].push(*b1idx);
            current_region_list[best_idx] = new_canidate_region;
        }
    }
    // moves vectors into result without re-allocations
    let final_assignment_points: Vec<_> = current_region_list
        .iter()
        .zip(final_assignment)
        .map(|(fin_region, assignments)| {
            (
                Point {
                    x: fin_region.x1,
                    y: fin_region.y1,
                },
                assignments,
            )
        })
        .collect();

    Ok(final_assignment_points)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use rand::prelude::*;
    use rand::Rng;
    use rand_chacha::ChaCha8Rng;
    use rand_distr::{Distribution, Normal};

    fn generate_boxes(region_size: i32, num_boxes: i32, max_box_size: i32, seed: i32) -> Vec<Box> {
        let mut rng = ChaCha8Rng::seed_from_u64(seed as u64);
        let sampler = Normal::new(0.0, region_size as f64 / 3.0).unwrap();
        (0..num_boxes)
            .map(|_| Box {
                x1: sampler.sample(&mut rng) as i32,
                y1: sampler.sample(&mut rng) as i32,
                xs: rng.gen_range(1..max_box_size as u32),
                ys: rng.gen_range(1..max_box_size as u32),
            })
            .collect()
    }
    #[test]
    fn test_efficient_avg_val() {
        let mut tot_val = 0;
        let count = 25;
        for i in 0..count {
            let orig_boxes = generate_boxes(10000, 5000, 50, i);
            let tile_width = 500;
            let tile_height = 500;
            let results = efficient_coverage(&orig_boxes, tile_width, tile_height).unwrap();
            tot_val += results.len();
        }
        let average_count = tot_val as f64 / count as f64;
        // sanity check that we get some number of results
        assert!(average_count > 50.);
        assert!(average_count < 790.);
    }
    #[test]
    fn test_coverage_det_speed() {
        let mut orig_boxes: Vec<Box> = Vec::new();
        let dim_size = 2000;
        let box_size = 50;
        for x in 0..dim_size {
            for y in 0..dim_size {
                orig_boxes.push(Box {
                    x1: x * box_size,
                    y1: y * box_size,
                    xs: box_size as u32,
                    ys: box_size as u32,
                });
            }
        }
        let tile_width = 500;
        let tile_height = 500;
        let results = efficient_coverage(&orig_boxes, tile_width, tile_height).unwrap();
        assert_eq!(results.len(), 40000);
    }
    #[test]
    fn test_efficient_coverage() {
        let orig_boxes = generate_boxes(10000, 50000, 50, 42);
        let tile_width = 500;
        let tile_height = 500;
        let results = efficient_coverage(&orig_boxes, tile_width, tile_height).unwrap();

        // sanity check that we get some number of results
        assert!(results.len() > 50);

        // check that all boxes are included in exactly one tile
        let mut results_cover = vec![false; orig_boxes.len()];
        for (_p, idxs) in results.iter() {
            for idx in idxs.iter() {
                assert!(!results_cover[*idx as usize]);
                results_cover[*idx as usize] = true;
            }
        }
        assert!(results_cover.iter().all(|x| *x));

        // check that all boxes are full included in their tile, no partial overlaps
        for (p, idxs) in results.iter() {
            for idx in idxs.iter() {
                let b = &orig_boxes[*idx as usize];
                assert!(
                    p.x <= b.x1
                        && p.y <= b.y1
                        && b.x1 - p.x <= (tile_width - b.xs) as i32
                        && b.y1 - p.y <= (tile_height - b.ys) as i32
                );
            }
        }
    }
    #[test]
    fn test_bad_box() {
        let orig_boxes = vec![
            Box {
                x1: -2995,
                y1: 501,
                xs: 100,
                ys: 52,
            },
            Box {
                x1: -25,
                y1: 51,
                xs: 1000,
                ys: 52,
            },
        ];
        let tile_width = 500;
        let tile_height = 500;

        let result = efficient_coverage(&orig_boxes, tile_width, tile_height);
        assert_eq!(
            result.err().unwrap().as_str(),
            "Box has size: (1000, 52), could not fit in tile of size: (500, 500)"
        )
    }
    #[test]
    fn test_empty() {
        let orig_boxes = Vec::new();
        let result = efficient_coverage(&orig_boxes, 500, 500);
        assert_eq!(result.unwrap().len(), 0);
    }
}
