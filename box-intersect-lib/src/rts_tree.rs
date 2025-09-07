use crate::box_def::Box;
use std::cmp::{max, min};

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
fn to_box(b: &CalcBox) -> Box {
    Box {
        x1: b.x1,
        xs: (b.x2 - b.x1) as u32,
        y1: b.y1,
        ys: (b.y2 - b.y1) as u32,
    }
}
const MAX_BIN_SIZE: usize = 8; // max size of tree leaves
const GOAL_NODE_SIZE: usize = 6; // rough size of nodes, must be smaller than MIN_BIN_SIZE

fn does_intersect(b1: &CalcBox, b2: &CalcBox) -> bool {
    b1.y2 > b2.y1 && b1.y1 < b2.y2 && b1.x2 > b2.x1 && b1.x1 < b2.x2
}
// Flipper is a type-level utility for recursive box flipping for compile-time box flipping
trait SortAxis {
    type Transpose: SortAxis;
    fn left(b: &CalcBox) -> i32;
    fn right(b: &CalcBox) -> i32;
}
struct YAxis {}
struct XAxis {}
impl SortAxis for YAxis {
    type Transpose = XAxis;
    fn left(b: &CalcBox) -> i32 {
        b.y1
    }
    fn right(b: &CalcBox) -> i32 {
        b.y2
    }
}
impl SortAxis for XAxis {
    type Transpose = YAxis;
    fn left(b: &CalcBox) -> i32 {
        b.x1
    }
    fn right(b: &CalcBox) -> i32 {
        b.x2
    }
}
enum SubNode {
    Children(Vec<RTSNode>),
    Data(Vec<(CalcBox, u32)>),
}
pub struct RTSNode {
    left: i32,
    right: i32,
    children: SubNode,
}
impl RTSNode {
    pub fn new(boxes: &[Box]) -> RTSNode {
        assert!(
            (boxes.len() as u64) < ((1_u64) << 32),
            "Only supports 4 billion boxes"
        );
        RTSNode::build_node::<XAxis>(
            boxes
                .iter()
                .enumerate()
                .map(|(idx, b)| (to_calc_box(b), idx as u32))
                .collect(),
        )
    }
    fn build_node<Axis: SortAxis>(mut boxes: Vec<(CalcBox, u32)>) -> RTSNode {
        let (parent_left, parent_right) =
            boxes
                .iter()
                .fold((i32::MAX, i32::MIN), |(left, right), (b, _)| {
                    (
                        min(Axis::Transpose::left(b), left),
                        max(Axis::Transpose::right(b), right),
                    )
                });
        // if data is small, brute force search is fastest, don't continue splitting, just optimize in place
        if boxes.len() <= MAX_BIN_SIZE {
            RTSNode {
                left: parent_left,
                right: parent_right,
                children: SubNode::Data(boxes),
            }
        } else {
            boxes.sort_unstable_by_key(|(b, _)| Axis::left(b));

            //divide the tree up in bins of at least GOAL_NODE_SIZE in size
            let child_sizes = boxes.len().div_ceil(GOAL_NODE_SIZE);
            let n_divs = boxes.len() / child_sizes;

            let mut children = Vec::<RTSNode>::with_capacity(n_divs);
            for i in 0..n_divs {
                let lidx = i * child_sizes;
                let ridx = if i == n_divs - 1 {
                    boxes.len()
                } else {
                    (i + 1) * child_sizes
                };
                let child_boxes = boxes[lidx..ridx].to_vec();
                children.push(RTSNode::build_node::<Axis::Transpose>(child_boxes))
            }
            RTSNode {
                left: parent_left,
                right: parent_right,
                children: SubNode::Children(children),
            }
        }
    }
    fn search_visitor_cb<F, Axis>(&self, orig_b1: &CalcBox, vistor: &mut F)
    where
        F: FnMut(&u32, &Box),
        Axis: SortAxis,
    {
        match &self.children {
            SubNode::Data(data) => {
                for (b2, idx) in data.iter() {
                    if does_intersect(orig_b1, b2) {
                        vistor(idx, &to_box(b2));
                    }
                }
            }
            SubNode::Children(children) => {
                for child in children.iter() {
                    if child.right > Axis::left(orig_b1) && child.left < Axis::right(orig_b1) {
                        child.search_visitor_cb::<F, Axis::Transpose>(orig_b1, vistor);
                    }
                }
            }
        }
    }
    pub fn search_visitor<F>(&self, b1: &Box, vistor: &mut F)
    where
        F: FnMut(&u32, &Box),
    {
        self.search_visitor_cb::<F, XAxis>(&to_calc_box(b1), vistor);
    }
    pub fn find_intersections(&self, b1: &Box) -> Vec<u32> {
        let mut v: Vec<u32> = Vec::new();
        self.search_visitor(b1, &mut |idx, _| {
            v.push(*idx);
        });
        v
    }
    pub fn tiled_order_visitor<F>(&self, vistor: &mut F)
    where
        F: FnMut(&u32, &Box),
    {
        match &self.children {
            SubNode::Data(data) => {
                for (b2, idx) in data.iter() {
                    vistor(idx, &to_box(b2));
                }
            }
            SubNode::Children(children) => {
                for child in children.iter() {
                    child.tiled_order_visitor(vistor);
                }
            }
        }
    }
}
/*
Tested in find_intersecting_asym, find_intersecting
*/
