use std::sync::Mutex;

use crate::api::{
    processes::{self, NativeHandle},
    sdk::{java, FromNative},
};

use super::{
    entity::{Vec2, Vec3, Vec4},
    java::JavaBuffer,
    minecraft::find_class,
    JClass,
};

pub fn multiply(vec: Vec4, mat: &Vec<f32>) -> Vec4 {
    Vec4 {
        x: vec.x * mat[0] as f64
            + vec.y * mat[4] as f64
            + vec.z * mat[8] as f64
            + vec.w * mat[12] as f64,
        y: vec.x * mat[1] as f64
            + vec.y * mat[5] as f64
            + vec.z * mat[9] as f64
            + vec.w * mat[13] as f64,
        z: vec.x * mat[2] as f64
            + vec.y * mat[6] as f64
            + vec.z * mat[10] as f64
            + vec.w * mat[14] as f64,
        w: vec.x * mat[3] as f64
            + vec.y * mat[7] as f64
            + vec.z * mat[11] as f64
            + vec.w * mat[15] as f64,
    }
}

pub fn world_to_screen(
    point: Vec3,
    out: &mut Vec2,
    model_view: &Vec<f32>,
    projection: &Vec<f32>,
    viewport: &Vec<i32>,
) -> bool {
    let clip_space_position = multiply(
        multiply(
            Vec4 {
                x: point.x,
                y: point.y,
                z: point.z,
                w: 1.0f64,
            },
            model_view,
        ),
        projection,
    );

    let ndc_space_pos = Vec3 {
        x: clip_space_position.x / clip_space_position.w,
        y: clip_space_position.y / clip_space_position.w,
        z: clip_space_position.z / clip_space_position.w,
    };

    if ndc_space_pos.z < -1.0f64 || ndc_space_pos.z > 1.0f64 {
        return false;
    }

    out.x = ((ndc_space_pos.x + 1.0f64) / 2.0f64) * viewport[2] as f64;
    out.y = ((1.0f64 - ndc_space_pos.y) / 2.0f64) * viewport[3] as f64;

    true
}

pub struct RenderInfo {
    activerenderinfo: JClass,
    rendermanager: JClass,
}

lazy_static::lazy_static! {
    static ref VIEWPORT_OFFSET: Mutex<usize> = Mutex::new(0usize);
    static ref MODELVIEW_OFFSET: Mutex<usize> = Mutex::new(0usize);
    static ref PROJECTION_OFFSET: Mutex<usize> = Mutex::new(0usize);
    static ref RENDERPOS_X_OFFSET: Mutex<usize> = Mutex::new(0usize);
}

impl RenderInfo {
    pub fn new() -> Self {
        Self {
            activerenderinfo: find_class("baj").clone(),
            rendermanager: find_class("bnn").clone(),
        }
    }

    pub fn get_viewport(&self, handle: &NativeHandle) -> JavaBuffer<i32> {
        if *VIEWPORT_OFFSET.lock().unwrap() == 0usize {
            *VIEWPORT_OFFSET.lock().unwrap() = self
                .activerenderinfo
                .find_field_entry(handle, "i", "Ljava/nio/IntBuffer;")
                .expect("Couldn't find viewport field entry")
                ._field_info
                .offset() as usize;
        }

        let mut viewport_pointer: u32 = 0;
        processes::read(
            &handle,
            self.activerenderinfo.static_fields as usize + *VIEWPORT_OFFSET.lock().unwrap(),
            &mut viewport_pointer,
        );

        java::JavaBuffer::from_native(&handle, viewport_pointer as *mut java::JavaBuffer<i32>)
    }

    pub fn get_modelview(&self, handle: &NativeHandle) -> JavaBuffer<f32> {
        if *MODELVIEW_OFFSET.lock().unwrap() == 0usize {
            *MODELVIEW_OFFSET.lock().unwrap() = self
                .activerenderinfo
                .find_field_entry(handle, "j", "Ljava/nio/FloatBuffer;")
                .expect("Couldn't find modelview field entry")
                ._field_info
                .offset() as usize;
        }

        let mut modelview_pointer: u32 = 0;
        processes::read(
            &handle,
            self.activerenderinfo.static_fields as usize + *MODELVIEW_OFFSET.lock().unwrap(),
            &mut modelview_pointer,
        );

        java::JavaBuffer::from_native(&handle, modelview_pointer as *mut java::JavaBuffer<f32>)
    }

    pub fn get_projection(&self, handle: &NativeHandle) -> JavaBuffer<f32> {
        if *PROJECTION_OFFSET.lock().unwrap() == 0usize {
            *PROJECTION_OFFSET.lock().unwrap() = self
                .activerenderinfo
                .find_field_entry(handle, "k", "Ljava/nio/FloatBuffer;")
                .expect("Couldn't find modelview field entry")
                ._field_info
                .offset() as usize;
        }

        let mut projection_pointer: u32 = 0;
        processes::read(
            handle,
            self.activerenderinfo.static_fields as usize + *PROJECTION_OFFSET.lock().unwrap(),
            &mut projection_pointer,
        );

        java::JavaBuffer::from_native(&handle, projection_pointer as *mut java::JavaBuffer<f32>)
    }

    pub fn get_render_position(&self, handle: &NativeHandle) -> Vec3 {
        if *RENDERPOS_X_OFFSET.lock().unwrap() == 0usize {
            *RENDERPOS_X_OFFSET.lock().unwrap() = self
                .rendermanager
                .find_field_entry(handle, "b", "D")
                .expect("Couldn't find renderPosX  field entry")
                ._field_info
                .offset() as usize;
        }

        let mut positioning: Vec3 = unsafe {
            Vec3 {
                ..core::mem::zeroed()
            }
        };

        processes::read(
            handle,
            self.rendermanager.static_fields as usize + *RENDERPOS_X_OFFSET.lock().unwrap(),
            &mut positioning,
        );

        positioning
    }
}
