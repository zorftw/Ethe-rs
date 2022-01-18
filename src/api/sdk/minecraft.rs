use crate::api::processes::{self, NativeHandle};

use super::{JClass, world::World, entity::Entity};
use crate::ether::CLASSES;

// Remote minecraft object
#[derive(Debug)]
pub struct Minecraft {
    _clazz: JClass,
    pub _address: usize, // pointer to the object
}


pub fn find_class(name: &str) -> JClass {
    let classes = CLASSES.lock().unwrap();

    classes.get(name).expect("Couldn't find class").clone()
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

    pub fn get_world(&self, handle: &NativeHandle) -> World {
        World::new(&find_class("bjf"), self.get_world_pointer(handle))
    }

    #[allow(unused)]
    pub fn get_player(&self, handle: &NativeHandle) -> Entity {
        Entity::new(self.get_player_pointer(handle))
    }

    #[allow(unused)]
    pub fn get_player_pointer(&self, handle: &NativeHandle) -> u32 {
        let mut result: u32 = 0;

        processes::read(
            &handle,
            self._address
                + self._clazz
                    .find_field_entry(&handle, "h", "Lbjk;")
                    .expect("Couldn't find player object field...")
                    ._field_info
                    .offset() as usize,
            &mut result,
        );

        result
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
