use std::{ops::Mul, usize};

use winapi::um::memoryapi::ReadProcessMemory;

use crate::api::processes::{self, NativeHandle};

use super::FromNative;

#[repr(C)]
#[derive(Debug)]
pub struct JavaBuffer<T> {
    padding_0000: [u8; 16],
    array: i32, // "address of the native array"
    padding_0001: [u8; 12],
    pub length: i32, // length of the buffer
    base: *mut Self, // address of ourselves
}

#[repr(C)]
#[derive(Debug)]
pub struct JavaArray<T> {
    pad_0000: [u8; 16],
    pub length: i32, // length of the array
    pub array: i32,  // pointer to actual list
    base: *mut Self,
}

impl<T> JavaArray<T> {
    pub fn array_offset(&self) -> i32 {
        0x10
    }

    pub fn get_at(&self, handle: &NativeHandle, idx: i32) -> Option<T> {
        if idx > self.length || idx < 0 {
            return None;
        }

        let address =
            (self.array + self.array_offset() + idx.mul(std::mem::size_of::<T>() as i32)) as u32;

        Some(processes::read_exact::<T>(&handle, address as usize))
    }
}

impl<T> JavaBuffer<T> {
    #[allow(dead_code)]
    pub fn get(&self, handle: &NativeHandle, idx: i32) -> Option<T> {
        if idx > self.length || idx < 0 {
            return None;
        }

        let address = unsafe { (self.array as *mut T).offset(idx as _) };
        Some(processes::read_exact::<T>(&handle, address as usize))
    }

    pub fn as_vec(&self, handle: &NativeHandle) -> Vec<T> {
        let mut res: Vec<T> = Vec::with_capacity(self.length as _);

        unsafe {
            res.set_len(self.length as _);
            ReadProcessMemory(
                handle.get(),
                self.array as _,
                res.as_mut_ptr() as _,
                self.length as usize * std::mem::size_of::<T>() as usize,
                std::ptr::null_mut(),
            );
        }

        res
    }
}

impl<T> FromNative for JavaBuffer<T> {
    fn from_native(handle: &NativeHandle, ptr: *mut Self) -> Self {
        let mut buffer =  processes::read_class::<JavaBuffer<T>>(handle, ptr as _);
        buffer.base = ptr;

        buffer
    }
}

impl<T> FromNative for JavaArray<T> {
    fn from_native(handle: &NativeHandle, ptr: *mut Self) -> Self {
        let mut buffer = processes::read_class::<JavaArray<T>>(handle, ptr as _);
        buffer.base = ptr;

        buffer
    }
}
