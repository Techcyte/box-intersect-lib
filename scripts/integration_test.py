"""
Very simple integration test, not meant to test for accuracy, that is left to the unit tests
"""
import box_intersect_lib_py
import numpy as np

# input is numpy array of boxes.
# boxes are left-anchored (x,y,width,height)
boxes = np.array([
    [1,1,1,2],
    [0,0,5,5],
    [2,3,2,6],
],dtype="int32")

results = box_intersect_lib_py.find_intersecting_boxes(boxes)

# 2nd box intersects with other 2
assert np.equal(results[0], np.array([1])).all()
assert np.equal(results[1], np.array([0,2])).all()
assert np.equal(results[2], np.array([1])).all()


# get area of the relevant intersections
intersection_areas = [box_intersect_lib_py.intersect_area(boxes[i], boxes[results[i]]) for i in range(len(boxes))]
assert np.equal(intersection_areas[0], np.array([2.])).all()
assert np.equal(intersection_areas[1], np.array([2., 4.])).all()
assert np.equal(intersection_areas[2], np.array([4.])).all()
# print(intersection_areas)  # [array([2], dtype=uint64), array([2, 4], dtype=uint64), array([4], dtype=uint64)]


# we can also build a data structure of boxes for efficient querying with arbirary other boxes
detector = box_intersect_lib_py.BoxIntersector(boxes)
query_box = (0,0,2,2)
intersecting_idxs = detector.find_intersections(*query_box)
assert np.equal(intersecting_idxs, np.array([0, 1])).all()


results = box_intersect_lib_py.efficient_coverage(boxes, 10, 10)
assert len(results) == 1
assert results[0][0] == (0,-1)
assert np.equal(np.sort(results[0][1]), np.array([0, 1, 2], dtype=np.uint32)).all()
