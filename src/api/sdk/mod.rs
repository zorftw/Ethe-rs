use std::iter::FromIterator;

use winapi::um::memoryapi::ReadProcessMemory;

use super::processes::{self, NativeHandle};

pub trait FromNative {
    fn from_native(handle: &NativeHandle, ptr: *mut Self) -> Self;
}

#[repr(C)]
pub struct JVMDictionary {
    pub table_size: i32,
    pub entries: *mut *mut DictionaryEntry,
    pub no_clue_what_the_hell_this_is: *mut usize,
    pub free_entry: *mut usize,
    pub end_block: *mut usize,
    pub entry_size: i32,
    pub num_entries: i32,
}

#[repr(C)]
#[derive(Debug)]
pub struct DictionaryEntry {
    pub hash: u64,
    pub next: usize,
    pub klass: *mut JClass,
    pub loader: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct JSymbol {
    pub lenght: i16,
    pub identity: i16,
    padding_0000: [u8; 4],
    pub text: *mut u8, //ptr to array that contains the actual unicode text
}

#[repr(C)]
#[derive(Debug)]
pub struct JArray<T> {
    pub lenght: i32,
    pub data: [T; 1],
}

#[repr(C)]
#[derive(Debug)]
pub struct JClass {
    padding_0000: [u8; 8],
    pub layout_helper: i32,
    pub super_check_offset: i32,
    pub symbol: *mut JSymbol,
    pub secondary_super_cache: *mut JClass,
    pub secondary_super_array: *mut usize,
    padding_0001: [u8; 64],
    pub static_fields: *mut usize,
    pub super_klass: *mut JClass,
    pub sub_klass: *mut JClass,
    pub next_sibling: *mut JClass,
    pub next_link: *mut JClass,
    pub classloader_data: *mut usize,
    pub modifier_flags: i32,
    pub access_flags: i32,
    padding_0002: [u8; 56],
    pub constant_pool: *mut usize,
    padding_0003: [u8; 143],
    n0000081_f: i8,
    padding_0004: [u8; 16],
    pub methods: *mut usize,
    pub default_methods: *mut usize,
    pub _local_interfaces: *mut usize,
    pub _transitive_interfaces: *mut usize,
    pub _method_ordering: *mut usize,
    pub _default_vtable_indices: *mut usize,
    pub fields: *mut JArray<u16>,
} //Size: 0x01B0

impl FromNative for JClass {
    fn from_native(handle: &NativeHandle, ptr: *mut Self) -> Self {
        unsafe { (processes::read_class::<JClass>(handle, ptr as _).get() as *mut Self).read() }
    }
}

impl FromNative for JSymbol {
    fn from_native(handle: &NativeHandle, ptr: *mut Self) -> Self {
        unsafe { (processes::read_class::<JSymbol>(handle, ptr as _).get() as *mut Self).read() }
    }
}

impl<T> FromNative for JArray<T> {
    fn from_native(handle: &NativeHandle, ptr: *mut Self) -> Self {
        unsafe { (processes::read_class::<JArray<T>>(handle, ptr as _).get() as *mut Self).read() }
    }
}

impl<T> JArray<T> {
    pub fn at(&self, i: i32) -> Option<T> {
        if i >= 0 && i < self.lenght {
            return Some(unsafe { self.data.as_ptr().offset(i as _).read() });
        }

        None
    }

    pub fn is_empty(&self) -> bool {
        self.lenght == 0
    }

    pub fn adr_at(&self, i: i32) -> *const T {
        if i >= 0 && i < self.lenght {
            return unsafe { self.data.as_ptr().offset(i as _) as *const T };
        }

        std::ptr::null()
    }
}

impl JSymbol {
    pub fn to_string(&self, ptr: usize, handle: &NativeHandle) -> String {
        // buffer for our string
        // note that these strings don't seem to have an end denominator?
        let mut buffer: Vec<u8> = Vec::new();
        buffer.resize(self.lenght as _, 0);

        unsafe {
            ReadProcessMemory(
                handle.get(),
                (ptr + 0x0008) as _,
                buffer.as_mut_ptr() as _,
                buffer.len(),
                std::ptr::null_mut(),
            );

            String::from_utf8_lossy(buffer.as_slice()).to_string()
        }
    }
}

impl DictionaryEntry {
    pub fn next(&self) -> usize {
        self.next & 0xFFFFFFFFFFFFFFFE
    }
}

impl Default for JVMDictionary {
    fn default() -> Self {
        Self {
            table_size: 0,
            entries: std::ptr::null_mut(),
            no_clue_what_the_hell_this_is: std::ptr::null_mut(),
            free_entry: std::ptr::null_mut(),
            end_block: std::ptr::null_mut(),
            entry_size: 0,
            num_entries: 0,
        }
    }
}

unsafe impl Send for JVMDictionary {}
