use std::vec;

use winapi::um::{
    errhandlingapi::GetLastError,
    handleapi::CloseHandle,
    memoryapi::{ReadProcessMemory, VirtualAlloc, VirtualFree},
    processthreadsapi::OpenProcess,
    tlhelp32::{
        CreateToolhelp32Snapshot, Module32First, Module32Next, Process32First, Process32Next,
        MODULEENTRY32, PROCESSENTRY32, TH32CS_SNAPMODULE, TH32CS_SNAPPROCESS,
    },
    winnt::{HANDLE, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE, PROCESS_ALL_ACCESS},
};

/// Structure to handle native handles
pub struct NativeHandle {
    _handle: HANDLE,
}

/// Native implementation of memory allocation, this was made so that we can allocate big chunks of memory inside our own program  (outside of the program heap)
/// this was causing issues with the Rust compiler not being able to allocate enough memory
pub struct NativeAllocation {
    /// Pointer to our base address
    _memory: *mut u8,

    /// Size of memory region
    _size: usize,
}

#[derive(Debug, Default)]
pub struct ProcessEntry {
    pub name: String,
    pub pid: u32,
}

#[derive(Debug, Default)]
pub struct ModuleEntry {
    pub name: String,
    pub base: usize,
    pub size: usize,
}

impl NativeAllocation {
    pub fn new(size: usize) -> Self {
        Self {
            _memory: unsafe { VirtualAlloc(std::ptr::null_mut(), size, MEM_COMMIT, PAGE_READWRITE) }
                as _,
            _size: size,
        }
    }

    pub fn get(&self) -> *mut u8 {
        self._memory
    }

    pub fn size(&self) -> usize {
        self._size
    }
}

impl NativeHandle {
    pub fn new(handle: HANDLE) -> Self {
        Self { _handle: handle }
    }

    pub fn get(&self) -> HANDLE {
        self._handle
    }
}

impl Drop for NativeAllocation {
    fn drop(&mut self) {
        unsafe {
            VirtualFree(self._memory as _, self._size, MEM_RELEASE);
        }
    }
}

impl Drop for NativeHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self._handle);
        }
    }
}

pub fn iterate_modules(process_id: u32) -> Vec<ModuleEntry> {
    let mut modules = vec![];

    unsafe {
        let snapshot: HANDLE = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, process_id);

        let module_entry: *mut MODULEENTRY32 =
            [0 as u8; std::mem::size_of::<MODULEENTRY32>()].as_mut_ptr() as *mut _;
        (*module_entry).dwSize = std::mem::size_of::<MODULEENTRY32>() as u32;

        if Module32First(snapshot, module_entry) == 1 {
            loop {
                modules.push(ModuleEntry {
                    name: String::from_utf8(
                        module_entry
                            .read()
                            .szModule
                            .iter()
                            .map(|i| *i as u8)
                            .take_while(|&i| i as char != char::from(0))
                            .collect(),
                    )
                    .unwrap_or_default(),
                    base: module_entry.read().modBaseAddr as usize,
                    size: module_entry.read().modBaseSize as usize,
                });

                if Module32Next(snapshot, module_entry) == 0 {
                    break;
                }
            }
        } else {
            println!("Error code: {}", GetLastError());
        }

        CloseHandle(snapshot);
    }

    modules
}

pub fn iterate_processes() -> Vec<ProcessEntry> {
    let mut processes = vec![];

    unsafe {
        let process_entry: *mut PROCESSENTRY32 =
            [0 as u8; std::mem::size_of::<PROCESSENTRY32>()].as_mut_ptr() as *mut _;
        (*process_entry).dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);

        if Process32First(snapshot, process_entry) == 1 {
            loop {
                processes.push(ProcessEntry {
                    name: String::from_utf8(
                        process_entry
                            .read()
                            .szExeFile
                            .iter()
                            .map(|i| *i as u8)
                            .take_while(|&i| i as char != char::from(0))
                            .collect(),
                    )
                    .unwrap_or_default(),
                    pid: process_entry.read().th32ProcessID,
                });

                if Process32Next(snapshot, process_entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
    }

    processes
}

pub fn read<T>(handle: &NativeHandle, address: usize, result: &mut T) {
    unsafe {
        ReadProcessMemory(
            handle.get(),
            address as _,
            result as *mut _ as _,
            std::mem::size_of::<T>(),
            std::ptr::null_mut(),
        );
    }
}

pub fn read_exact<T>(handle: &NativeHandle, address: usize) -> NativeAllocation {
    let buffer = NativeAllocation::new(std::mem::size_of::<T>());

    unsafe {
        ReadProcessMemory(
            handle.get(),
            address as _,
            buffer.get() as _,
            buffer.size(),
            std::ptr::null_mut(),
        );
    }

    buffer
}

pub fn read_class<T>(handle: &NativeHandle, address: usize) -> NativeAllocation {
    //let buffer: *mut T = vec![0 as i8; std::mem::size_of::<T>()].as_mut_ptr() as *mut _;
    let buffer = NativeAllocation::new(std::mem::size_of::<T>());

    unsafe {
        ReadProcessMemory(
            handle.get(),
            address as _,
            buffer.get() as _,
            buffer._size,
            std::ptr::null_mut(),
        );
    }

    buffer
}

pub fn find_process(name: &str) -> Option<ProcessEntry> {
    iterate_processes().into_iter().find(|p| p.name.eq(name))
}

pub fn find_module(name: &str, process: Option<ProcessEntry>) -> Option<ModuleEntry> {
    iterate_modules(process.unwrap_or(ProcessEntry::default()).pid)
        .into_iter()
        .find(|m| m.name.eq(name))
}

pub fn open_process(entry: &ProcessEntry) -> Option<NativeHandle> {
    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, 0, entry.pid);

        if handle as usize != 0x0 {
            return Some(NativeHandle::new(handle as HANDLE));
        }
    }

    None
}
