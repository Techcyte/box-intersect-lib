use crate::box_def::Box;
use crate::intersect_calc::does_intersect;
use crate::rts_tree::RTSNode;

/*
 * Some tests suggest that using CalcBox over Box gives a ~30% performance boost.
 */
#[derive(Clone, Copy)]
struct CalcBox {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}
fn to_calc_box(b: &Box) -> CalcBox {
    CalcBox {
        x1: b.x1,
        x2: b.x1 + b.xs as i32,
        y1: b.y1,
        y2: b.y1 + b.ys as i32,
    }
}

pub fn find_intersecting_boxes_linesearch(raw_boxes: &[Box]) -> Vec<Vec<u32>> {
    assert!(
        (raw_boxes.len() as u64) < ((1_u64) << 32),
        "Only supports 4 billion boxes"
    );
    let mut sorted_boxes: Vec<(CalcBox, u32)> = raw_boxes
        .iter()
        .enumerate()
        .map(|(idx, b)| (to_calc_box(b), idx as u32))
        .collect();
    sorted_boxes.sort_unstable_by_key(|(b, _)| b.x1);

    // only stores boxes to the left ot itself
    let nodes_to_left: Vec<Vec<u32>> = sorted_boxes
        .iter()
        .enumerate()
        .map(|(sidx, (b1, _))| {
            sorted_boxes[sidx + 1..]
                .iter()
                .take_while(|(b2, _)| b2.x1 < b1.x2)
                .filter(|(b2, _)| b1.y2 > b2.y1 && b1.y1 < b2.y2)
                .map(|(_, item)| *item)
                .collect()
        })
        .collect();
    let mut graph = vec![Vec::new(); raw_boxes.len()];
    // mirror over all directed connections so that boxes intersect with those to the right of themselves
    for (l, (_, bidx)) in nodes_to_left.iter().zip(sorted_boxes.iter()) {
        for i in l.iter() {
            graph[*i as usize].push(*bidx);
            graph[*bidx as usize].push(*i);
        }
    }
    graph
}

pub fn find_intersecting_boxes_rts(boxes: &[Box]) -> Vec<Vec<u32>> {
    let intersect_finder = RTSNode::new(boxes);
    let mut results = vec![Vec::<u32>::new(); boxes.len()];
    // iterating through the nodes in same order they are stored in the RTree gives 2x performance
    // on large data, due to superior data locality and branch prediction.
    // NOTE: this hurts performance in multi-threaded implementations, no clue why
    // boxes.iter().enumerate().for_each(&mut |(idx, b1)| {
    //     let idx1 = &(idx as u32);
    intersect_finder.tiled_order_visitor(&mut |idx1, b1| {
        let mut v: Vec<u32> = Vec::new();
        intersect_finder.search_visitor(b1, &mut |idx2, _| {
            if *idx1 != *idx2 {
                v.push(*idx2);
            }
        });
        results[*idx1 as usize] = v;
    });
    results
}

pub fn find_intersecting_boxes_all_cmp(boxes: &[Box]) -> Vec<Vec<u32>> {
    boxes
        .iter()
        .enumerate()
        .map(|(idx1, b1)| {
            boxes
                .iter()
                .enumerate()
                .filter(|(idx2, b2)| idx1 != *idx2 && does_intersect(b1, b2))
                .map(|(idx, _)| idx as u32)
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use rand::Rng;
    use rand_distr::StandardNormal;
    fn generate_boxes(region_size: i32, num_boxes: i32, max_box_size: i32) -> Vec<Box> {
        let mut rng = rand::thread_rng();
        (0..num_boxes)
            .map(|_| {
                let x1f: f64 = rng.sample(StandardNormal);
                let y1f: f64 = rng.sample(StandardNormal);
                Box {
                    x1: (x1f * region_size as f64) as i32,
                    y1: (y1f * region_size as f64) as i32,
                    xs: rng.gen_range(1..max_box_size as u32),
                    ys: rng.gen_range(1..max_box_size as u32),
                }
            })
            .collect()
    }
    fn sort_graph(mut graph: Vec<Vec<u32>>) -> Vec<Vec<u32>> {
        for edges in graph.iter_mut() {
            edges.sort();
        }
        graph
    }
    #[test]
    fn test_boxes_linesearch_acc() {
        let boxes = generate_boxes(500, 10000, 100);
        let gold_result = find_intersecting_boxes_all_cmp(&boxes);
        let test_result = sort_graph(find_intersecting_boxes_linesearch(&boxes));
        let num_intersections: usize = test_result.iter().map(|l| l.len()).sum();
        assert_eq!(gold_result, test_result);
        //sanity check
        assert!(num_intersections > 0);
    }
    #[test]
    fn test_boxes_rts_acc() {
        let boxes = generate_boxes(500, 10000, 100);
        let gold_result = find_intersecting_boxes_all_cmp(&boxes);
        let test_result = sort_graph(find_intersecting_boxes_rts(&boxes));
        let num_intersections: usize = test_result.iter().map(|l| l.len()).sum();
        assert_eq!(gold_result, test_result);
        //sanity check
        assert!(num_intersections > 0);
    }
    #[test]
    fn test_boxes_rts_zero_length_array() {
        let boxes = generate_boxes(500, 0, 100);
        let test_result = sort_graph(find_intersecting_boxes_rts(&boxes));
        let num_intersections: usize = test_result.iter().map(|l| l.len()).sum();
        //sanity check
        assert!(num_intersections == 0);
    }
}
