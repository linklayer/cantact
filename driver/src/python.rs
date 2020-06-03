use crate::Frame;
use crate::Interface as RustInterface;
use cpython::*;

impl ToPyObject for Frame {
    type ObjectType = PyDict;
    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        let dict = PyDict::new(py);
        dict.set_item(py, "id", self.can_id).unwrap();
        dict.set_item(py, "dlc", self.can_dlc).unwrap();
        dict.set_item(py, "channel", self.channel).unwrap();
        dict.set_item(py, "data", PyBytes::new(py, &self.data))
            .unwrap();
        dict
    }
}
/* TODO
impl FromPyObject for Frame {
    fn extract(py: Python, obj: &PyObject) -> PyResult<Self> {
        Frame{}
    }
}
*/

py_class!(class Interface |py| {
    data i: RustInterface;
    def __new__(_cls) -> PyResult<Interface> {
        let i = RustInterface::new();
        Interface::create_instance(py, i)
    }

    def start(&self, channel: u16) -> PyResult<bool> {
        self.i(py).start(channel);
        Ok(true)
    }

    def stop(&self, channel: u16) -> PyResult<bool> {
        self.i(py).stop(channel);
        Ok(true)
    }

    def set_bitrate(&self, channel: u16, bitrate: u32) -> PyResult<bool> {
        self.i(py).set_bitrate(channel, bitrate);
        Ok(true)
    }

    def recv(&self) -> PyResult<Frame> {
        let f = self.i(py).recv().unwrap();
        Ok(f)
    }
/* TODO
    def send(&self, f: Frame) -> PyResult<Frame> {
        let f = self.i(py).send(f).unwrap();
        Ok(f)
    }
*/
});

py_module_initializer!(cantact, initcantact, PyInit_cantact, |py, m| {
    m.add(
        py,
        "__doc__",
        "A Python wrapper for the CANtact USB library",
    )
    .unwrap();
    m.add_class::<Interface>(py)?;
    Ok(())
});
