from ast import arg
import numpy as np
import argparse
import box_intersect_lib_py
import timeit


def generate_data(num_boxes, region_size, max_box_size):
    assert max_box_size > 0 and num_boxes > 0 and region_size > 0
    assert region_size > max_box_size, "Region size must be larger than the max box size"
    box_positions = np.random.randint(low=0, high=region_size-max_box_size, size=(num_boxes, 2), dtype="int32")
    box_sizes = np.random.randint(low=1, high=max_box_size, size=(num_boxes, 2), dtype="int32")
    boxes = np.concatenate([box_positions, box_sizes], axis=1)
    return boxes



def main():
    parser = argparse.ArgumentParser(description='Run a benchmark on a variety of proceedures in box-intersect-lib using python interface.')
    parser.add_argument('--num-boxes', default=200000, type=int, help='Number of boxes in input.')
    parser.add_argument('--region-size', default=80000, type=int, help='Region size that boxes are randomly placed in. Smaller regions mean more intersections.')
    parser.add_argument('--max-box-size', default=100, type=int, help='Boxes have uniformly random size 1 to max-box-size. Larger max size means more intersections')
    parser.add_argument('--test-iterations', default=2, type=int, help='Number of iterations to perform for benchmarking. Higher means more accurate and longer runtimes.')
    args = parser.parse_args()

    data = generate_data(args.num_boxes, args.region_size, args.max_box_size)
    float_data = np.random.uniform(size=args.num_boxes)
    #data2 = generate_data(args.num_boxes, args.region_size, args.max_box_size)
    num_itersections = sum(len(l) for l in box_intersect_lib_py.find_intersecting_boxes(data))

    find_intersecting_boxes_t = timeit.timeit('box_intersect_lib_py.find_intersecting_boxes(data)', number=args.test_iterations, globals={**globals(),**locals()})/args.test_iterations
    find_non_max_suppressed_t = timeit.timeit('box_intersect_lib_py.find_non_max_suppressed(data,float_data,0.1,0.3)', number=args.test_iterations, globals={**globals(),**locals()})/args.test_iterations
    find_intersecting_boxes_asym_t = timeit.timeit('box_intersect_lib_py.find_intersecting_boxes_asym(data, data)', number=args.test_iterations, globals={**globals(),**locals()})/args.test_iterations
    find_best_matches_t = timeit.timeit('box_intersect_lib_py.find_best_matches(data, data, 0.1)', number=args.test_iterations, globals={**globals(),**locals()})/args.test_iterations
    find_rect_cover_t = timeit.timeit('box_intersect_lib_py.efficient_coverage(data, 500, 500)', number=args.test_iterations, globals={**globals(),**locals()})/args.test_iterations
    find_intersecting_boxes_ls_t = timeit.timeit('box_intersect_lib_py.find_intersecting_boxes_linesearch(data)', number=args.test_iterations, globals={**globals(),**locals()})/args.test_iterations
    find_intersecting_boxes_build_intersector_t = timeit.timeit('box_intersect_lib_py.BoxIntersector(data)', number=args.test_iterations, globals={**globals(),**locals()})/args.test_iterations
    find_intersecting_boxes_build_query_intersector_t = timeit.timeit('detect = box_intersect_lib_py.BoxIntersector(data);\nfor b in data[:len(data)//10]: detect.find_intersections(*b)', number=max(1,args.test_iterations//4), globals={**globals(),**locals()})*10/max(1,args.test_iterations//4)
    print("num boxes|num intersections| find_intersecting_boxes_t|find_non_max_suppressed_t|find_intersecting_boxes_linesearch_t|find_intersecting_boxes_asym_t|find_best_matches_t|find_rect_cover_t|BoxIntersector build|BoxIntersector query sequentially")
    print("---|---|---|---|---|---|---|---|---|---")
    print(f"{args.num_boxes}|{num_itersections}|{find_intersecting_boxes_t}|{find_non_max_suppressed_t}|{find_intersecting_boxes_ls_t}|{find_intersecting_boxes_asym_t}|{find_best_matches_t}|{find_rect_cover_t}|{find_intersecting_boxes_build_intersector_t}|{find_intersecting_boxes_build_query_intersector_t}")


if __name__ == "__main__":
    main()