use crate::Error;
use crate::{Frame, Interface};
use crossbeam_channel::{unbounded, Receiver, RecvTimeoutError, Sender};
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass(name = Interface)]
struct PyInterface {
    i: Interface,
    rx_recv: Receiver<Frame>,
    rx_send: Sender<Frame>,
}
impl IntoPy<PyObject> for Frame {
    fn into_py(self, py: Python) -> PyObject {
        let d = PyDict::new(py);
        d.set_item("id", self.can_id).unwrap();
        d.set_item("dlc", self.can_dlc).unwrap();
        d.set_item("data", self.data.to_vec()).unwrap();
        d.set_item("extended", self.ext).unwrap();
        d.set_item("rtr", self.rtr).unwrap();
        d.set_item("channel", self.channel).unwrap();
        d.to_object(py)
    }
}

impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        PyErr::new::<exceptions::SystemError, _>(format!("{:?}", err))
    }
}

#[pymethods]
impl PyInterface {
    #[new]
    fn new() -> PyResult<Self> {
        let mut i = Interface::new()?;

        // disable all channels by default
        for n in 0..i.channels.len() {
            i.set_enabled(n, false)?;
        }

        let (send, recv) = unbounded();
        Ok(PyInterface {
            i: i,
            rx_recv: recv,
            rx_send: send,
        })
    }

    fn set_bitrate(&mut self, channel: usize, bitrate: u32) -> PyResult<()> {
        self.i.set_bitrate(channel, bitrate)?;
        Ok(())
    }

    fn set_enabled(&mut self, channel: usize, enabled: bool) -> PyResult<()> {
        self.i.set_enabled(channel, enabled)?;
        Ok(())
    }

    fn set_monitor(&mut self, channel: usize, enabled: bool) -> PyResult<()> {
        self.i.set_monitor(channel, enabled)?;
        Ok(())
    }

    fn start(&mut self) -> PyResult<()> {
        let rx = self.rx_send.clone();

        self.i.start(move |f: Frame| {
            match rx.send(f) {
                Ok(_) => {}
                Err(_) => { /*TODO*/ }
            };
        })?;

        Ok(())
    }

    fn stop(&mut self) -> PyResult<()> {
        self.i.stop()?;
        Ok(())
    }

    fn recv(&self, timeout_ms: u64) -> PyResult<Option<Frame>> {
        let f = match self
            .rx_recv
            .recv_timeout(std::time::Duration::from_millis(timeout_ms))
        {
            Ok(f) => f,
            Err(RecvTimeoutError::Timeout) => return Ok(None),
            Err(RecvTimeoutError::Disconnected) => panic!("device thread died"),
        };
        Ok(Some(f))
    }

    fn send(
        &mut self,
        channel: u8,
        id: u32,
        ext: bool,
        rtr: bool,
        dlc: u8,
        data: Vec<u8>,
    ) -> PyResult<()> {
        let mut data_array = [0u8; 8];
        for i in 0..dlc as usize {
            data_array[i] = data[i];
        }
        self.i.send(Frame {
            can_id: id,
            can_dlc: dlc,
            ext: ext,
            rtr: rtr,
            data: data_array,
            channel: channel,
            loopback: false,
            fd: false,
        })?;
        Ok(())
    }

    fn channel_count(&self) -> PyResult<usize> {
        Ok(self.i.channels())
    }
}

#[pymodule]
fn cantact(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyInterface>()?;
    Ok(())
}
