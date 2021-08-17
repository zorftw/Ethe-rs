use crate::api::processes::{self, NativeHandle};

use super::JClass;

// Remote minecraft object
#[derive(Debug)]
pub struct Minecraft {
    _clazz: JClass,
    pub _address: usize, // pointer to the object
}

impl Minecraft {
    pub fn new(class: &JClass, handle: &NativeHandle) -> Self {
        let mut address: u32 = 0;
        processes::read(
            &handle,
            class.static_fields as usize
                + class
                    .find_field_entry(&handle, "M", "Lbao;")
                    .expect("Couldn't find minecraft object field...")
                    ._field_info
                    .offset() as usize,
            &mut address,
        );

        Self {
            _clazz: class.clone(),
            _address: address as _,
        }
    }

    pub fn get_world_pointer(&self, handle: &NativeHandle) -> u32 {
        let mut result: u32 = 0;

        processes::read(
            &handle,
            self._address
                + self._clazz
                    .find_field_entry(&handle, "f", "Lbjf;")
                    .expect("Couldn't find world object field...")
                    ._field_info
                    .offset() as usize,
            &mut result,
        );

        result
    }
}
