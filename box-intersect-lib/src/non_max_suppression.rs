use crate::box_def::Box;
use crate::intersect_calc::*;
use crate::rts_tree::RTSNode;

fn overlap_meets_threshold(b1: &Box, b2: &Box, iou_threshold: f64, overlap_threshold: f64) -> bool {
    let intersect = intersect_area(b1, b2) as f64;
    let union_a = union_area(b1, b2) as f64;
    intersect > union_a * iou_threshold || intersect > b2.area() as f64 * overlap_threshold
}

pub fn find_non_max_suppressed(
    all_boxes: &[Box],
    scores: &[f64],
    iou_threshold: f64,
    overlap_threshold: f64,
) -> Vec<bool> {
    assert_eq!(all_boxes.len(), scores.len());
    let intersect_finder = RTSNode::new(all_boxes);

    let mut mask = vec![true; scores.len()];
    let mut local_max = vec![false; scores.len()];
    let mut score_idxs: Vec<_> = scores.iter().copied().enumerate().collect();
    //sorts in decreasing order of score
    score_idxs.sort_unstable_by(|(_, s1), (_, s2)| {
        s1.partial_cmp(s2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .reverse()
    });
    for (idx, _) in score_idxs.iter() {
        if mask[*idx] {
            local_max[*idx] = true;
            let b1 = all_boxes[*idx];
            intersect_finder.search_visitor(&b1, &mut |idx2, b2| {
                if overlap_meets_threshold(&b1, b2, iou_threshold, overlap_threshold) {
                    mask[*idx2 as usize] = false;
                }
            });
        }
    }
    local_max
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use rand::Rng;
    use rand_distr::{Distribution, Normal};
    use std::cmp::Ordering;
    pub fn find_non_max_suppressed_gold(
        all_boxes: &[Box],
        scores: &[f64],
        iou_threshold: f64,
        overlap_threshold: f64,
    ) -> Vec<bool> {
        let mut sorted_items: Vec<(f64, Box, usize)> = all_boxes
            .iter()
            .zip(scores.iter())
            .enumerate()
            .map(|(idx, (b, s))| (*s, *b, idx))
            .collect();
        sorted_items
            .sort_by(|(s1, _, _), (s2, _, _)| s1.partial_cmp(s2).unwrap_or(Ordering::Equal));
        sorted_items.reverse();
        let mut suppressed = vec![false; all_boxes.len()];
        for (sidx1, (s1, b1, idx1)) in sorted_items.iter().enumerate() {
            if !suppressed[*idx1] {
                for (_, (s2, b2, idx2)) in sorted_items[sidx1 + 1..].iter().enumerate() {
                    if !suppressed[*idx2]
                        && *s1 > *s2 // in theory not necessary due to sorting
                        && (overlap_meets_threshold(b1, &b2, iou_threshold, overlap_threshold))
                    {
                        suppressed[*idx2] = true;
                    }
                }
            }
        }
        let non_suppressed = suppressed.iter().map(|s| !*s).collect();
        non_suppressed
    }

    #[allow(dead_code)]
    fn generate_boxes(region_size: i32, num_boxes: i32, max_box_size: i32) -> Vec<Box> {
        let mut rng = rand::thread_rng();
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
    fn test_nms_acc() {
        let mut rng = rand::thread_rng();
        let region_size = 1000;
        let num_boxes = 10000;
        let max_box_size = 100;
        let boxes: Vec<Box> = (0..num_boxes)
            .map(|_| Box {
                x1: rng.gen_range(0..region_size - max_box_size),
                y1: rng.gen_range(0..region_size - max_box_size),
                xs: rng.gen_range(1..max_box_size as u32),
                ys: rng.gen_range(1..max_box_size as u32),
            })
            .collect();
        let scores: Vec<f64> = (0..num_boxes).map(|_| rng.gen()).collect();
        let gold_result = find_non_max_suppressed_gold(&boxes, &scores, 0.1, 0.5);
        let test_result = find_non_max_suppressed(&boxes, &scores, 0.1, 0.5);
        let num_survivors: usize = gold_result.iter().map(|x| if *x { 1 } else { 0 }).sum();
        assert_eq!(gold_result, test_result);
        //sanity check on gold impl
        assert!(num_survivors > 5 && num_survivors < num_boxes - 5);
    }
}
