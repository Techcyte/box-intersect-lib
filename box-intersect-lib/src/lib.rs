mod box_def;
mod efficient_coverage;
mod find_intersecting;
mod find_intersecting_asym;
mod intersect_calc;
mod match_boxes;
mod non_max_suppression;
mod rts_tree;

pub use crate::box_def::Box;
pub use crate::efficient_coverage::{efficient_coverage, Point};
pub use crate::find_intersecting::{
    find_intersecting_boxes_all_cmp, find_intersecting_boxes_linesearch,
    find_intersecting_boxes_rts,
};
pub use crate::find_intersecting_asym::find_intersecting_boxes_asym;
pub use crate::intersect_calc::{does_intersect, intersect_area, union_area};
pub use crate::match_boxes::find_best_matches;
pub use crate::non_max_suppression::find_non_max_suppressed;
pub use crate::rts_tree::RTSNode;
