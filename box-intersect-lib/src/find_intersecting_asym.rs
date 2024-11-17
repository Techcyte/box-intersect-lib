use crate::box_def::Box;
use crate::rts_tree::RTSNode;

pub fn find_intersecting_boxes_asym(boxes_src: &[Box], boxes_dest: &[Box]) -> Vec<Vec<u32>> {
    /* Returns a directed bipartite graph from src to dest */
    let intersect_finder = RTSNode::new(boxes_dest);
    boxes_src
        .iter()
        .map(|b1| intersect_finder.find_intersections(b1))
        .collect()
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::intersect_calc::does_intersect;
    use rand::Rng;

    fn find_intersecting_boxes_asym_gold(boxes_src: &[Box], boxes_dest: &[Box]) -> Vec<Vec<u32>> {
        boxes_src
            .iter()
            .map(|b1| {
                boxes_dest
                    .iter()
                    .enumerate()
                    .filter(|(_, b2)| does_intersect(b1, b2))
                    .map(|(idx, _)| idx as u32)
                    .collect()
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
    fn test_boxes_asym_acc() {
        let mut rng = rand::thread_rng();
        let region_size = 2000;
        let num_boxes = 10000;
        let max_box_size = 100;
        let boxes_src: Vec<Box> = (0..num_boxes)
            .map(|_| Box {
                x1: rng.gen_range(0..region_size - max_box_size),
                y1: rng.gen_range(0..region_size - max_box_size),
                xs: rng.gen_range(1..max_box_size as u32),
                ys: rng.gen_range(1..max_box_size as u32),
            })
            .collect();
        let boxes_dest: Vec<Box> = (0..num_boxes)
            .map(|_| Box {
                x1: rng.gen_range(0..region_size - max_box_size),
                y1: rng.gen_range(0..region_size - max_box_size),
                xs: rng.gen_range(1..max_box_size as u32),
                ys: rng.gen_range(1..max_box_size as u32),
            })
            .collect();
        let gold_result = find_intersecting_boxes_asym_gold(&boxes_src, &boxes_dest);
        let test_result = sort_graph(find_intersecting_boxes_asym(&boxes_src, &boxes_dest));
        let num_intersections: usize = test_result.iter().map(|l| l.len()).sum();
        assert_eq!(gold_result, test_result);
        //sanity check
        assert!(num_intersections > 0);
    }

    #[test]
    fn test_boxes_rts_zero_length_array() {
        let mut rng = rand::thread_rng();
        let region_size = 2000;
        let num_boxes = 100;
        let max_box_size = 100;
        let boxes_dest: Vec<Box> = (0..num_boxes)
            .map(|_| Box {
                x1: rng.gen_range(0..region_size - max_box_size),
                y1: rng.gen_range(0..region_size - max_box_size),
                xs: rng.gen_range(1..max_box_size as u32),
                ys: rng.gen_range(1..max_box_size as u32),
            })
            .collect();
        let boxes_src: Vec<Box> = Vec::new();
        let test_result = sort_graph(find_intersecting_boxes_asym(&boxes_src, &boxes_dest));
        let num_intersections: usize = test_result.iter().map(|l| l.len()).sum();
        //sanity check
        assert!(num_intersections == 0);
    }
}
