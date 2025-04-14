use crate::nmea_message;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyModule};
use pyo3::wrap_pyfunction;

#[pymodule]
fn nmea(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Message>()?;
    Ok(())
}

#[pyclass]
struct Message {
    inner: nmea_message::Message,
}

#[pymethods]
impl Message {
    #[new]
    fn new() -> Self {
        Self {
            inner: nmea_message::Message::new(),
        }
    }

    fn add_frame(&mut self, payload: &[u8]) -> PyResult<bool> {
        // Enforce 8 byte payload as input to add_frame is [u8; 8].
        if payload.len() != 8 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Payload must be exactly 8 bytes",
            ));
        }
        let payload_array: [u8; 8] = payload.try_into().unwrap();
        self.inner
            .add_frame(&payload_array)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    #[staticmethod]
    fn from_payload(payload: &[u8], sequence_counter: u8) -> Self {
        Self {
            inner: nmea_message::Message::from_payload(payload, sequence_counter),
        }
    }

    fn pop_frame(&mut self) -> Option<PyObject> {
        let frame = self.inner.pop_frame();
        match frame {
            Some(frame) => {
                let data: [u8; 8] = frame.bytes.try_into().unwrap();
                Some(Python::with_gil(|py| PyBytes::new(py, &data).to_object(py)))
            }
            None => None,
        }
    }

    fn get_payload(&mut self) -> PyResult<PyObject> {
        let mut buf: [u8; nmea_message::MAX_NMEA_PACKET_SIZE] =
            [0xFF; nmea_message::MAX_NMEA_PACKET_SIZE];
        let len = self.inner.get_payload(&mut buf);
        Python::with_gil(|py| Ok(PyBytes::new(py, &buf[..len]).to_object(py)))
    }

    fn clear(&mut self) {
        self.inner.clear();
    }

    #[getter]
    fn num_frames(&self) -> u8 {
        self.inner.num_frames
    }

    #[getter]
    fn sequence_counter(&self) -> u8 {
        self.inner.sequence_counter
    }

    #[getter]
    fn data_len(&self) -> u8 {
        self.inner.data_len
    }
}
