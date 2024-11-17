use crate::box_def::Box;
use crate::intersect_calc::{intersect_area, union_area};
use crate::rts_tree::RTSNode;

struct Intersection {
    pub iou: f64,
    pub idx1: u32,
    pub idx2: u32,
}

pub fn find_best_matches(
    rects1: &[Box],
    rects2: &[Box],
    iou_threshold: f64,
) -> (Vec<(u32, u32)>, Vec<u32>, Vec<u32>) {
    let rect1_checker = RTSNode::new(rects1);
    let mut intersections: Vec<Intersection> = Vec::new();
    for (r2idx, rect2) in rects2.iter().enumerate() {
        rect1_checker.search_visitor(rect2, &mut |r1idx, rect1| {
            let i_area = intersect_area(rect1, rect2) as f64;
            let u_area = union_area(rect1, rect2) as f64;
            if u_area > 0. && u_area * iou_threshold <= i_area {
                let iou = i_area / u_area;
                intersections.push(Intersection {
                    iou,
                    idx1: *r1idx,
                    idx2: r2idx as u32,
                });
            }
        });
    }
    // keeping this as a stable sort will make ties predictable
    intersections.sort_by(|a, b| b.iou.partial_cmp(&a.iou).unwrap());

    let mut unmatched_r1: Vec<bool> = vec![true; rects1.len()];
    let mut unmatched_r2: Vec<bool> = vec![true; rects2.len()];
    let mut matches: Vec<(u32, u32)> = Vec::new();
    for inter in intersections.iter() {
        if unmatched_r1[inter.idx1 as usize] && unmatched_r2[inter.idx2 as usize] {
            matches.push((inter.idx1, inter.idx2));
            unmatched_r1[inter.idx1 as usize] = false;
            unmatched_r2[inter.idx2 as usize] = false;
        }
    }
    let rem_r1: Vec<u32> = unmatched_r1
        .iter()
        .enumerate()
        .filter(|(_idx, val)| **val)
        .map(|(idx, _)| idx as u32)
        .collect();
    let rem_r2: Vec<u32> = unmatched_r2
        .iter()
        .enumerate()
        .filter(|(_idx, val)| **val)
        .map(|(idx, _)| idx as u32)
        .collect();

    (matches, rem_r1, rem_r2)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[test]
    fn test_find_best_matches_ordering() {
        let rects1 = [
            Box {
                x1: 1,
                y1: 3,
                xs: 3,
                ys: 3,
            },
            Box {
                x1: 1,
                y1: 2,
                xs: 3,
                ys: 3,
            },
            Box {
                x1: 1,
                y1: 1,
                xs: 3,
                ys: 3,
            },
        ];
        let rects2 = [
            Box {
                x1: 1,
                y1: 0,
                xs: 4,
                ys: 4,
            },
            Box {
                x1: 1,
                y1: 4,
                xs: 4,
                ys: 4,
            },
        ];
        let iou_threshold = 0.1;
        let (matches, rem_r1, rem_r2) = find_best_matches(&rects1, &rects2, iou_threshold);
        let expected_matches: Vec<(u32, u32)> = vec![(2, 0), (0, 1)];
        let expected_rem_r1: Vec<u32> = vec![1];
        let expected_rem_r2: Vec<u32> = vec![];
        assert_eq!(matches, expected_matches);
        assert_eq!(rem_r1, expected_rem_r1);
        assert_eq!(rem_r2, expected_rem_r2);
    }
    #[test]
    fn test_find_best_matches_ordering_reversed() {
        //same as test_find_best_matches_ordering, but with rects1 and rects2 swapped
        let rects1 = [
            Box {
                x1: 1,
                y1: 0,
                xs: 4,
                ys: 4,
            },
            Box {
                x1: 1,
                y1: 4,
                xs: 4,
                ys: 4,
            },
        ];
        let rects2 = [
            Box {
                x1: 1,
                y1: 3,
                xs: 3,
                ys: 3,
            },
            Box {
                x1: 1,
                y1: 2,
                xs: 3,
                ys: 3,
            },
            Box {
                x1: 1,
                y1: 1,
                xs: 3,
                ys: 3,
            },
        ];
        let iou_threshold = 0.1;
        let (matches, rem_r1, rem_r2) = find_best_matches(&rects1, &rects2, iou_threshold);
        let expected_matches: Vec<(u32, u32)> = vec![(0, 2), (1, 0)];
        let expected_rem_r1: Vec<u32> = vec![];
        let expected_rem_r2: Vec<u32> = vec![1];
        assert_eq!(matches, expected_matches);
        assert_eq!(rem_r1, expected_rem_r1);
        assert_eq!(rem_r2, expected_rem_r2);
    }
    #[test]
    fn test_find_best_matches_thresold() {
        //same input as test_find_best_matches_ordering, but with higher threshold
        let rects1 = [
            Box {
                x1: 1,
                y1: 3,
                xs: 3,
                ys: 3,
            },
            Box {
                x1: 1,
                y1: 2,
                xs: 3,
                ys: 3,
            },
            Box {
                x1: 1,
                y1: 1,
                xs: 3,
                ys: 3,
            },
        ];
        let rects2 = [
            Box {
                x1: 1,
                y1: 0,
                xs: 4,
                ys: 4,
            },
            Box {
                x1: 1,
                y1: 4,
                xs: 4,
                ys: 4,
            },
        ];
        let iou_threshold = 0.5;
        let (matches, rem_r1, rem_r2) = find_best_matches(&rects1, &rects2, iou_threshold);
        let expected_matches: Vec<(u32, u32)> = vec![(2, 0)];
        let expected_rem_r1: Vec<u32> = vec![0, 1];
        let expected_rem_r2: Vec<u32> = vec![1];
        assert_eq!(matches, expected_matches);
        assert_eq!(rem_r1, expected_rem_r1);
        assert_eq!(rem_r2, expected_rem_r2);
    }
}
