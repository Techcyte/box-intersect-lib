use box_intersect_lib::Box;
use numpy::PyArrayMethods;
use numpy::{PyArray, PyArray1, PyArray2, PyReadonlyArray1, PyReadonlyArray2};
use pyo3::exceptions::PyAssertionError;
use pyo3::prelude::*;
use pyo3::types::PyList;


type BoundU32Array2<'py> = pyo3::Bound<'py, PyArray2<u32>>;
type BoundU32Array1<'py> = pyo3::Bound<'py, PyArray1<u32>>;


fn generate_boxes<Func>(array: &PyReadonlyArray2<i32>, func: &mut Func) -> PyResult<()>
where
    Func: FnMut(Box),
{
    let dims = array.dims();
    let lenx = dims[1];
    let leny = dims[0];
    if lenx != 4 {
        return Err(PyAssertionError::new_err(
            "Expects 2nd dimension of box array to be size 4 (x, y, width, height)",
        ));
    }
    if leny == 0 {
        return Ok(());
    }
    let arr = array.as_array();
    let contig_arr = arr.as_standard_layout();
    let slice = contig_arr.as_slice().unwrap();
    for i in 0..leny {
        let barr = &slice[i * 4..(i + 1) * 4];
        let (x1, y1, width, height) = (barr[0], barr[1], barr[2], barr[3]);
        if width <= 0 || height <= 0 {
            return Err(PyAssertionError::new_err(
                "Expects width and hight of boxes to be greater than 0",
            ));
        }
        func(Box {
            x1,
            y1,
            xs: width as u32,
            ys: height as u32,
        });
    }
    Ok(())
}

fn np_arr_to_boxes(array: &PyReadonlyArray2<i32>) -> PyResult<Vec<Box>> {
    let mut boxes: Vec<Box> = Vec::with_capacity(array.dims()[0]);
    generate_boxes(array, &mut |b| {
        boxes.push(b);
    })?;
    Ok(boxes)
}

fn np_arr_to_box(array: &PyReadonlyArray1<i32>) -> PyResult<Box> {
    let dims = array.dims();
    let len = dims[0];
    if len != 4 {
        return Err(PyAssertionError::new_err(
            "Expects box array to be size 4 (x, y, width, height)",
        ));
    }
    let arr = array.as_array();
    if arr[2] <= 0 || arr[3] <= 0 {
        return Err(PyAssertionError::new_err(
            "Expects width and hight of boxes to be greater than 0",
        ));
    }
    let box_ = Box {
        x1: arr[0],
        y1: arr[1],
        xs: arr[2] as u32,
        ys: arr[3] as u32,
    };
    Ok(box_)
}

fn adj_list_to_py_list(py: Python<'_>, adj_list: Vec<Vec<u32>>) -> PyResult<Bound<'_, PyList>> {
    let mut list:Vec<Bound<'_, PyList>> = Vec::new();
    for l in adj_list {
        let pyl = PyList::new(py, l.clone())?;
        list.push(pyl);
    }
    PyList::new(
        py,
        list,
    )
}

#[pyfunction]
fn find_intersecting_boxes_rts<'py>(
    py: Python<'py>,
    boxes_array: PyReadonlyArray2<i32>,
) -> PyResult<Bound<'py, PyList>> {
    let boxes = np_arr_to_boxes(&boxes_array)?;
    let adj_list = Python::detach(py, move || box_intersect_lib::find_intersecting_boxes_rts(&boxes));
    adj_list_to_py_list(py, adj_list)
}

#[pyfunction]
fn find_intersecting_boxes_linesearch<'py>(
    py: Python<'py>,
    boxes_array: PyReadonlyArray2<i32>,
) -> PyResult<Bound<'py, PyList>> {
    let boxes = np_arr_to_boxes(&boxes_array)?;
    let adj_list =
        Python::detach(py, move || box_intersect_lib::find_intersecting_boxes_linesearch(&boxes));
    adj_list_to_py_list(py, adj_list)
}

#[pyfunction]
fn find_intersecting_boxes<'py>(
    py: Python<'py>,
    boxes_array: PyReadonlyArray2<i32>,
) -> PyResult<Bound<'py, PyList>> {
    find_intersecting_boxes_rts(py, boxes_array)
}

#[pyfunction]
fn find_intersecting_boxes_asym<'py>(
    py: Python<'py>,
    boxes_array_src: PyReadonlyArray2<i32>,
    boxes_array_dest: PyReadonlyArray2<i32>,
) -> PyResult<Bound<'py, PyList>> {
    let boxes_vec_src = np_arr_to_boxes(&boxes_array_src)?;
    let boxes_vec_dest = np_arr_to_boxes(&boxes_array_dest)?;
    let adj_list = Python::detach(py, move || {
        box_intersect_lib::find_intersecting_boxes_asym(&boxes_vec_src, &boxes_vec_dest)
    });
    adj_list_to_py_list(py, adj_list)
}

#[pyfunction]
fn find_best_matches<'py>(
    py: Python<'py>,
    boxes_array_1: PyReadonlyArray2<i32>,
    boxes_array_2: PyReadonlyArray2<i32>,
    iou_threshold: f64,
) -> PyResult<(
    BoundU32Array2<'py>,
    BoundU32Array1<'py>,
    BoundU32Array1<'py>,
)> {
    let boxes_vec_1 = np_arr_to_boxes(&boxes_array_1)?;
    let boxes_vec_2 = np_arr_to_boxes(&boxes_array_2)?;
    let (matches, rem1, rem2) = Python::detach(py, move || {
        box_intersect_lib::find_best_matches(&boxes_vec_1, &boxes_vec_2, iou_threshold)
    });
    let flat_matches_vec: Vec<u32> = matches.iter().flat_map(|x| [x.0, x.1]).collect();
    let np_matches = PyArray::from_vec(py, flat_matches_vec).reshape([matches.len(), 2])?;
    Ok((
        np_matches,
        PyArray::from_vec(py, rem1),
        PyArray::from_vec(py, rem2),
    ))
}

#[pyfunction]
fn intersect_area<'py>(
    py: Python<'py>,
    box1: PyReadonlyArray1<i32>,
    boxes: PyReadonlyArray2<i32>,
) -> PyResult<pyo3::Bound<'py, PyArray1<u64>>> {
    let box_ = np_arr_to_box(&box1)?;
    let mut area_vec: Vec<u64> = Vec::with_capacity(boxes.dims()[0]);
    generate_boxes(&boxes, &mut |b| {
        area_vec.push(box_intersect_lib::intersect_area(&box_, &b));
    })?;
    Ok(PyArray::from_vec(py, area_vec))
}

#[pyfunction]
fn area<'py>(
    py: Python<'py>,
    boxes: PyReadonlyArray2<i32>,
) -> PyResult<pyo3::Bound<'py, PyArray1<u64>>> {
    let mut area_vec: Vec<u64> = Vec::with_capacity(boxes.dims()[0]);
    generate_boxes(&boxes, &mut |b| {
        area_vec.push(b.area());
    })?;
    Ok(PyArray::from_vec(py, area_vec))
}

#[pyfunction]
fn find_non_max_suppressed<'py>(
    py: Python<'py>,
    boxes_array: PyReadonlyArray2<i32>,
    scores: PyReadonlyArray1<f64>,
    iou_threshold: f64,
    overlap_threshold: f64,
) -> PyResult<pyo3::Bound<'py, PyArray1<bool>>> {
    let boxes = np_arr_to_boxes(&boxes_array)?;
    let scores_arr = scores.as_array();
    let contig_arr = scores_arr.as_standard_layout();
    let scores_slice = contig_arr.as_slice().unwrap();
    if boxes.len() != scores_slice.len() {
        return Err(PyAssertionError::new_err(
            "Length of boxes list must match length of scores list",
        ));
    }
    let suppressed_mask = Python::detach(py, move || {
        box_intersect_lib::find_non_max_suppressed(
            &boxes,
            scores_slice,
            iou_threshold,
            overlap_threshold,
        )
    });
    Ok(PyArray::from_vec(py, suppressed_mask))
}

#[pyfunction]
fn efficient_coverage<'py>(
    py: Python<'py>,
    boxes_array: PyReadonlyArray2<i32>,
    tile_width: u32,
    tile_height: u32,
) -> PyResult<Vec<((i32, i32), BoundU32Array1<'py>)>> {
    let boxes = np_arr_to_boxes(&boxes_array)?;
    let results = Python::detach(py, move || {
            box_intersect_lib::efficient_coverage(&boxes, tile_width, tile_height)
        })
        .map_err(PyAssertionError::new_err)?;
    let py_results = results
        .iter()
        .map(|(p, intlist)| ((p.x, p.y), PyArray::from_vec(py, intlist.to_owned())))
        .collect();
    Ok(py_results)
}

#[pyclass]
struct BoxIntersector {
    inner: box_intersect_lib::RTSNode,
}

#[pymethods]
impl BoxIntersector {
    #[new]
    pub fn new(boxes_arr: PyReadonlyArray2<i32>) -> PyResult<Self> {
        Ok(BoxIntersector {
            inner: box_intersect_lib::RTSNode::new(&np_arr_to_boxes(&boxes_arr)?),
        })
    }
    pub fn find_intersections<'py>(
        &self,
        py: Python<'py>,
        x1: i32,
        y1: i32,
        width: u32,
        height: u32,
    ) -> PyResult<pyo3::Bound<'py, PyArray1<u32>>> {
        Ok(PyArray::from_vec(
            py,
            Python::detach(py, move || {
                self.inner.find_intersections(&Box {
                    x1,
                    y1,
                    xs: width,
                    ys: height,
                })
            }),
        ))
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn box_intersect_lib_py(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(find_intersecting_boxes_rts, m)?)?;
    m.add_function(wrap_pyfunction!(find_intersecting_boxes_linesearch, m)?)?;
    m.add_function(wrap_pyfunction!(find_intersecting_boxes_asym, m)?)?;
    m.add_function(wrap_pyfunction!(find_best_matches, m)?)?;
    m.add_function(wrap_pyfunction!(find_intersecting_boxes, m)?)?;
    m.add_function(wrap_pyfunction!(intersect_area, m)?)?;
    m.add_function(wrap_pyfunction!(area, m)?)?;
    m.add_function(wrap_pyfunction!(find_non_max_suppressed, m)?)?;
    m.add_function(wrap_pyfunction!(efficient_coverage, m)?)?;
    m.add_class::<BoxIntersector>()?;
    Ok(())
}
