#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod c {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

const COORD_FAC: f64 = 1e5;

fn mkc(p: f64) -> i64 { (p * COORD_FAC) as i64 }
fn mkm(p: i64) -> f64 { p as f64 / COORD_FAC }
unsafe fn diff(p: *mut c::IntPoint, q: *mut c::IntPoint) -> (f64, f64) {
    (mkm((*p).X - (*q).X), mkm((*p).Y - (*q).Y))
}

/// Wraps the ClipperOffset class to present a more or less Rustic interface.

#[pyclass(unsendable)]
pub struct Offseter {
    co: *mut c::ClipperOffset,
    pa: *mut c::Path,
    radius: f64,
    side: f64,
    mode: u32,
}

#[pymethods]
impl Offseter {
    #[new]
    pub fn new(arc_step: f64, radius: f64, side: f64, mode: &str) -> Self {
        unsafe {
            Self {
                co: c::cl_offset_new(2., 0.1 * arc_step * COORD_FAC),
                pa: c::cl_path_new(),
                radius, side,
                mode: if mode == "round" { c::jtRound } else { c::jtMiter }
            }
        }
    }

    pub fn add_point(&mut self, x: f64, y: f64) -> PyResult<()> {
        if self.pa.is_null() {
            return Err(PyRuntimeError::new_err("object is already used"));
        }
        unsafe {
            c::cl_path_add(self.pa, mkc(x), mkc(y));
        }
        Ok(())
    }

    #[allow(clippy::result_unit_err)]
    pub fn offset_shape(&mut self) -> PyResult<OffsetPoly> {
        if self.pa.is_null() {
            return Err(PyRuntimeError::new_err("object is already used"));
        }
        unsafe {
            let os = c::cl_path_get(self.pa, 0);
            let oe = c::cl_path_get(self.pa, c::cl_path_size(self.pa) - 1);
            let closed = ((*os).X - (*oe).X).abs() < 10 &&
                ((*os).Y - (*oe).Y).abs() < 10;

            let delta = self.radius * COORD_FAC *
                if c::cl_path_orientation(self.pa) { -self.side } else { self.side };
            let rp = c::cl_offset_path(self.co, self.pa, delta,
                                       self.mode, c::etClosedPolygon);
            if c::cl_paths_size(rp) != 1 {
                c::cl_paths_free(rp);
                Err(PyRuntimeError::new_err("no solution found"))
            } else {
                Ok(OffsetPoly {
                    closed, pi: std::mem::replace(&mut self.pa, std::ptr::null_mut()), po: rp,
                })
            }
        }
    }
}

impl Drop for Offseter {
    fn drop(&mut self) {
        unsafe {
            c::cl_offset_free(self.co);
            if !self.pa.is_null() {
                c::cl_path_free(self.pa);
            }
        }
    }
}

#[pyclass(unsendable)]
pub struct OffsetPoly {
    closed: bool,
    pi: *mut c::Path,
    po: *mut c::Paths,
}

#[pymethods]
impl OffsetPoly {
    pub fn fix_direction(&mut self, index: usize, invert: bool) -> usize {
        unsafe {
            let pin1 = c::cl_path_get(self.pi, 0);
            let pin2 = c::cl_path_get(self.pi, 1);

            let new_path = c::cl_paths_get(self.po, 0);
            let new_size = c::cl_path_size(new_path);
            let pout1 = c::cl_path_get(new_path, index as i32);
            let pout2 = c::cl_path_get(new_path, (index as i32 + 1) % new_size);
            let pout0 = c::cl_path_get(new_path, (index as i32 + new_size - 1) % new_size);

            let din = diff(pin2, pin1);
            let dout1 = diff(pout2, pout1);
            let dout2 = diff(pout0, pout1);

            let ain = din.1.atan2(din.0);
            let aout1 = dout1.1.atan2(dout1.0);
            let aout2 = dout2.1.atan2(dout2.0);
            let adiff1 = (ain - aout1).abs();
            let adiff2 = (ain - aout2).abs();

            if (adiff2 < adiff1) ^ invert {
                c::cl_path_reverse(new_path);
                new_size as usize - index - 1
            } else {
                index
            }
        }
    }

    pub fn first_point_delta(&self, index: usize) -> (f64, f64) {
        unsafe {
            let pos_in = c::cl_path_get(self.pi, 0);
            let new_path = c::cl_paths_get(self.po, 0);
            let pos_out = c::cl_path_get(new_path, index as i32);
            diff(pos_out, pos_in)
        }
    }

    pub fn closest_start(&self) -> usize {
        if !self.closed {
            return 0;
        }

        // find the starting point closest to the original's starting point
        let mut min_index = 0;
        let mut min_dist = i64::max_value();
        unsafe {
            let p1 = c::cl_path_get(self.pi, 0);
            let pos1 = ((*p1).X, (*p1).Y);

            let new_path = c::cl_paths_get(self.po, 0);
            let new_path_len = c::cl_path_size(new_path);
            assert!(c::cl_path_size(new_path) > 0);

            for i in 0..new_path_len {
                let p = c::cl_path_get(new_path, i);
                let pos = ((*p).X, (*p).Y);
                let dist = (pos1.0 - pos.0).pow(2) + (pos1.1 - pos.1).pow(2);
                if dist < min_dist {
                    min_dist = dist;
                    min_index = i;
                }
            }
        }
        min_index as usize
    }

    pub fn reconstruct(&self, start_at: usize, reconstruct: &Bound<PyAny>) -> PyResult<()> {
        unsafe {
            let start_at = start_at as i32;
            let new_path = c::cl_paths_get(self.po, 0);
            let new_path_len = c::cl_path_size(new_path);
            assert!(c::cl_path_size(new_path) > 0);

            let p = c::cl_path_get(new_path, start_at);
            let start_pos = (mkm((*p).X), mkm((*p).Y));

            let mut pos = start_pos;
            for i in 1..new_path_len {
                let p = c::cl_path_get(new_path, (start_at + i) % new_path_len);
                let end_pos = (mkm((*p).X), mkm((*p).Y));
                reconstruct.call((pos, end_pos), None)?;
                pos = end_pos;
            }
            if self.closed {
                reconstruct.call((pos, start_pos), None)?;
            }
        }
        Ok(())
    }
}


impl Drop for OffsetPoly {
    fn drop(&mut self) {
        unsafe {
            c::cl_path_free(self.pi);
            c::cl_paths_free(self.po);
        }
    }
}

#[pymodule]
fn clipppy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Offseter>()?;
    m.add_class::<OffsetPoly>()?;
    Ok(())
}
