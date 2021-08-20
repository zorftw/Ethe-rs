use std::sync::Mutex;

use crate::api::processes::{self, NativeHandle};

use super::{minecraft::find_class};

#[derive(Debug)]
pub struct Entity {
    pub _address: usize,
}

#[derive(Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug)]
pub struct Vec4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[derive(Debug)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec3 {
    #[allow(unused)]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

lazy_static::lazy_static! {
    static ref POSITION_OFFSET: Mutex<usize> = Mutex::new(0usize);
    static ref LAST_TICK_POSITION_OFFSET: Mutex<usize> = Mutex::new(0usize);
}

impl Entity {
    pub fn new(address: u32) -> Self {
        Self {
            _address: address as _,
        }
    }

    pub fn get_head_position(&self, handle: &NativeHandle) -> Vec3 {
        let pos = self.get_position(handle);

        Vec3 {
            x: pos.x,
            y: pos.y + 1.8f64,
            z: pos.z,
        }
    }

    pub fn get_last_tick_position(&self, handle: &NativeHandle) -> Vec3 {
        if *LAST_TICK_POSITION_OFFSET.lock().unwrap() == 0usize {
            let clazz = find_class("bll");
            *LAST_TICK_POSITION_OFFSET.lock().unwrap() = clazz
                .find_field_entry(&handle, "S", "D")
                .expect("Couldn't find lastTickPosX field")
                ._field_info
                .offset() as usize;
        }

        let mut positioning: Vec3 = unsafe {
            Vec3 {
                ..core::mem::zeroed()
            }
        };

        processes::read(
            &handle,
            (self._address + *LAST_TICK_POSITION_OFFSET.lock().unwrap()) as usize,
            &mut positioning,
        );

        positioning
    }

    pub fn get_position(&self, handle: &NativeHandle) -> Vec3 {
        if *POSITION_OFFSET.lock().unwrap() == 0usize {
            let clazz = find_class("bll");
            *POSITION_OFFSET.lock().unwrap() = clazz
                .find_field_entry(&handle, "s", "D")
                .expect("Couldn't find posX field")
                ._field_info
                .offset() as usize;
        }

        let mut positioning: Vec3 = unsafe {
            Vec3 {
                ..core::mem::zeroed()
            }
        };

        processes::read(
            &handle,
            (self._address + *POSITION_OFFSET.lock().unwrap()) as usize,
            &mut positioning,
        );

        positioning
    }
}
