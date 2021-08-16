use std::{iter::FromIterator, ops::Mul};

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
pub struct JFieldInfo {
    _shorts: [u16; 6],
}

pub enum JFieldOffset {
    AccessFlagsOffset = 0,
    NameIndexOffset,
    SignatureIndexOffset,
    InitvalIndexOffset,
    LowPackedOffset,
    HighPackedOffset,
    FieldSlots,
}

#[repr(C)]
#[derive(Debug)]
pub struct JArray<T> {
    pub lenght: i32,
    pub data: [T; 1],
    base: *mut Self,
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

pub struct FieldEntry {
    pub field_info: JFieldInfo,
}

impl FromNative for JFieldInfo {
    fn from_native(handle: &NativeHandle, ptr: *mut Self) -> Self {
        unsafe { (processes::read_class::<JFieldInfo>(handle, ptr as _).get() as *mut Self).read() }
    }
}

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
        let mut array = unsafe { (processes::read_class::<JArray<T>>(handle, ptr as _).get() as *mut Self).read() };
        array.base = ptr;

        array
    }
}

impl FieldEntry {
    pub fn new(jinfo: JFieldInfo) -> Self {
        Self {
            field_info: jinfo,
        }
    }
}

impl JClass {
    pub fn iterate_fields(&self, handle: &NativeHandle) -> impl Iterator<Item = FieldEntry> {
        let mut fields: Vec<FieldEntry> = Vec::new();
        let fields_array = JArray::from_native(&handle, self.fields);

        for i in 0..fields_array.lenght {
            if fields_array.adr_at(i * JFieldOffset::FieldSlots.value()) as usize == 0usize {
                continue;
            }

            let field_info = JFieldInfo::from_native(handle, fields_array.adr_at(i * JFieldOffset::FieldSlots.value()) as _);
            fields.push(FieldEntry::new(field_info));
        }

        fields.into_iter()
    }
}

impl JFieldOffset {
    pub fn value(&self) -> i32 {
        match *self {
            JFieldOffset::AccessFlagsOffset => 0,
            JFieldOffset::NameIndexOffset => 1,
            JFieldOffset::SignatureIndexOffset => 2,
            JFieldOffset::InitvalIndexOffset => 3,
            JFieldOffset::LowPackedOffset => 4,
            JFieldOffset::HighPackedOffset => 5,
            JFieldOffset::FieldSlots => 6,
        }
    }
}

impl JFieldInfo {
    fn build_int_from_shorts(&self, low: u16, high: u16) -> i32 {
        ((high << 16) | low) as _
    }

    pub fn offset(&self) -> u64 {
        (self.build_int_from_shorts(self._shorts[JFieldOffset::LowPackedOffset.value() as usize], self._shorts[JFieldOffset::HighPackedOffset.value() as usize]) >> 2) as _
    }
    
    pub fn has_offset(&self) -> bool {
        (self._shorts[JFieldOffset::LowPackedOffset.value() as usize] & (1 << 0)) != 0
    }

    pub fn name_idx(&self) -> u16 {
        self._shorts[JFieldOffset::NameIndexOffset.value() as usize]
    }

    pub fn sig_idx(&self) -> u16 {
        self._shorts[JFieldOffset::SignatureIndexOffset.value() as usize]
    }
}

impl<T> JArray<T> {
    pub fn at(&self, i: i32, handle: &NativeHandle) -> Option<T> {
        if i >= 0 && i < self.lenght {
            let result = processes::read_handeled::<T>(handle, self.base as usize + std::mem::size_of::<i32>() + std::mem::size_of::<T>().mul(i as usize));
            unsafe { return Some((result.get() as *mut T).read()); }
        }

        None
    }

    pub fn is_empty(&self) -> bool {
        self.lenght == 0
    }

    pub fn adr_at(&self, i: i32) -> *const T {
        if i >= 0 && i < self.lenght {
            //return unsafe { self.data.as_ptr().offset(i as _) as *const T };
            return (self.base as usize + std::mem::size_of::<i32>() + std::mem::size_of::<T>().mul(i as usize)) as _;
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
