use std::ops::Mul;

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
    pub base: *mut Self,
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
    pub constant_pool: *mut JConstantPool,
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

#[repr(C)]
#[derive(Debug)]
pub struct JConstantPool {
    padding_0000: [u8; 8],
    pub tags: *const JArray<u8>,
    pub cache: *const usize,
    pub instance_klass: *const JClass,
    pub operands: *const JArray<u16>,
    pub resolved: *const JArray<JClass>,
    pub major: u16,
    pub minor: u16,
    pub generic_signature_index: u16,
    pub source_file_name_index: u16,
    pub flags: u16,
    pub length: i32,
    pub saved: i32,
    lock: *const usize,
    base: *mut Self,
} //0x50

pub struct FieldEntry {
    pub _field_info: JFieldInfo,
    pub name: String,
    pub sig: String,
}

impl FromNative for JFieldInfo {
    fn from_native(handle: &NativeHandle, ptr: *mut Self) -> Self {
        unsafe { (processes::read_class::<JFieldInfo>(handle, ptr as _).get() as *mut Self).read() }
    }
}

impl FromNative for JConstantPool {
    fn from_native(handle: &NativeHandle, ptr: *mut Self) -> Self {
        let mut constant_pool = unsafe { (processes::read_class::<JConstantPool>(handle, ptr as _).get() as *mut Self).read() };
        constant_pool.base = ptr;

        constant_pool
    }
}

impl FromNative for JClass {
    fn from_native(handle: &NativeHandle, ptr: *mut Self) -> Self {
        unsafe { (processes::read_class::<JClass>(handle, ptr as _).get() as *mut Self).read() }
    }
}

impl FromNative for JSymbol {
    fn from_native(handle: &NativeHandle, ptr: *mut Self) -> Self {
        let mut symbol = unsafe { (processes::read_class::<JSymbol>(handle, ptr as _).get() as *mut Self).read() };
        symbol.base = ptr;

        symbol
    }
}

impl<T> FromNative for JArray<T> {
    fn from_native(handle: &NativeHandle, ptr: *mut Self) -> Self {
        let mut array = unsafe {
            (processes::read_class::<JArray<T>>(handle, ptr as _).get() as *mut Self).read()
        };
        array.base = ptr;

        array
    }
}

impl FieldEntry {
    pub fn new(jinfo: JFieldInfo, constant_pool: &JConstantPool, handle: &NativeHandle) -> Self {
        let name = constant_pool.symbol(&handle, jinfo.name_idx() as _).expect("Unable to get symbol");
        let signature = constant_pool.symbol(&handle, jinfo.sig_idx() as _).expect("Unable to get signature symbol");
        
        Self { 
            _field_info: jinfo,
            name: name.to_string(&handle),
            sig: signature.to_string(&handle),
        }
    }
}

impl JConstantPool {

    pub fn size(&self) -> usize {
        0x50
    }

    // pub fn symbol_offset(&self, handle: &NativeHandle, which: isize) -> usize {
    //     unsafe { ((self.base as usize + self.size()) as *mut *mut JSymbol).offset(which) as _}
    // }
    pub fn symbol(&self, handle: &NativeHandle, which: isize) -> Option<JSymbol> {
        let address = unsafe { ((self.base as usize + self.size()) as *mut *mut JSymbol).offset(which) as usize};
        let mut symbol_addy: usize = 0usize;

        processes::read(handle, address, &mut symbol_addy);

        if symbol_addy != 0usize {
            //return Some(processes::read_class::<JSymbol>(handle, symbol_addy).get() as *mut Jsy);
            return Some(JSymbol::from_native(&handle, symbol_addy as *mut _));
        }

        None
    }
}

impl JClass {
    pub fn iterate_fields(&self, handle: &NativeHandle) -> impl Iterator<Item = FieldEntry> {
        let mut fields: Vec<FieldEntry> = Vec::new();
        let fields_array = JArray::from_native(&handle, self.fields);

        let constant_pool = JConstantPool::from_native(&handle,self.constant_pool);

        for i in 0..fields_array.lenght {
            if fields_array.adr_at(i * JFieldOffset::FieldSlots.value()) as usize == 0usize {
                continue;
            }

            let field_info = JFieldInfo::from_native(
                handle,
                fields_array.adr_at(i * JFieldOffset::FieldSlots.value()) as _,
            );
            fields.push(FieldEntry::new(field_info, &constant_pool, handle));
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
        (((high as u32) << 16) | (low as u32)) as _
    }

    pub fn offset(&self) -> u64 {
        (self.build_int_from_shorts(
            self._shorts[JFieldOffset::LowPackedOffset.value() as usize],
            self._shorts[JFieldOffset::HighPackedOffset.value() as usize],
        ) >> 2) as _
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
            let result = processes::read_exact::<T>(
                handle,
                self.base as usize
                    + std::mem::size_of::<i32>()
                    + std::mem::size_of::<T>().mul(i as usize),
            );
            unsafe {
                return Some((result.get() as *mut T).read());
            }
        }

        None
    }

    pub fn is_empty(&self) -> bool {
        self.lenght == 0
    }

    pub fn adr_at(&self, i: i32) -> *const T {
        if i >= 0 && i < self.lenght {
            //return unsafe { self.data.as_ptr().offset(i as _) as *const T };
            return (self.base as usize
                + std::mem::size_of::<i32>()
                + std::mem::size_of::<T>().mul(i as usize)) as _;
        }

        std::ptr::null()
    }
}

impl JSymbol {
    pub fn to_string(&self, handle: &NativeHandle) -> String {
        // buffer for our string
        // note that these strings don't seem to have an end denominator?
        let mut buffer: Vec<u8> = Vec::new();
        buffer.resize(self.lenght as _, 0);

        unsafe {
            ReadProcessMemory(
                handle.get(),
                (self.base as usize + 0x0008) as _,
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
