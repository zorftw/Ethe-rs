#[cfg(not(windows))]
compile_error!("Ether-rs is exclusive to Windows at the moment");

// declare extern crate
extern crate winapi;

// our "api"
mod api;

use api::*;

#[cfg(windows)]
fn main() {
    use winapi::um::{handleapi::CloseHandle, memoryapi::ReadProcessMemory};

    println!("Ethe-rs is Ether but Rust, because Rust owns me and all");

    let dictionary_pattern = sig::Signature::new("48 8b 0d ?? ?? ?? ?? 4c 8b cd 44 8b c7");

    println!("Searching for: {:?}", dictionary_pattern.to_bytes());

    if let Some(javaw) = processes::find_process("javaw.exe") {
        if let Some(handle) = processes::open_process(&javaw) {
            if let Some(jvm_dll) = processes::find_module("jvm.dll", Some(javaw)) {
                println!("Module jvm.dll at address {:p}", jvm_dll.base as *mut i8);

                let dictionary = sig::pattern_scan_module(handle, &dictionary_pattern, jvm_dll)
                    .as_mut()
                    .and_then(|dictionary| {
                        let mut offset: i32 = 0;
                        processes::read(handle, *dictionary + 3, &mut offset);
                        let end = *dictionary + 7 + offset as usize;

                        let dictionary_buffer: *mut sdk::JVMDictionary =
                            [0 as i8; std::mem::size_of::<sdk::JVMDictionary>()].as_mut_ptr()
                                as *mut _;
                        let mut address: i32 = 0;
                        processes::read(handle, end, &mut address);

                        unsafe {
                            ReadProcessMemory(
                                handle,
                                address as _,
                                dictionary_buffer as *mut _,
                                std::mem::size_of::<sdk::JVMDictionary>(),
                                std::ptr::null_mut(),
                            );
                            return Some(dictionary_buffer.read());
                        };
                    })
                    .unwrap();

                for i in 0..dictionary.table_size {
                    let dictionary_entry: *mut sdk::DictionaryEntry =
                        [0 as i8; std::mem::size_of::<sdk::DictionaryEntry>()].as_mut_ptr()
                            as *mut _;

                    unsafe {
                        ReadProcessMemory(
                            handle,
                            (dictionary.entries.offset(i as _)) as *mut _,
                            dictionary_entry as *mut _,
                            std::mem::size_of::<sdk::DictionaryEntry>(),
                            std::ptr::null_mut(),
                        );

                        while dictionary_entry != std::ptr::null_mut() {
                            if dictionary_entry.read().klass != std::ptr::null_mut() {
                                println!("Klass: {:p}", dictionary_entry.read().klass);
                            }

                            if dictionary_entry.read().next() == 0
                                || dictionary_entry.read().klass == std::ptr::null_mut()
                            {
                                break;
                            }

                            ReadProcessMemory(
                                handle,
                                dictionary_entry.read().next() as *mut _,
                                dictionary_entry as *mut _,
                                std::mem::size_of::<sdk::DictionaryEntry>(),
                                std::ptr::null_mut(),
                            );
                        }
                    }
                }
            } else {
                println!("Couldn't find address of jvm.dll :(");
                std::thread::sleep(std::time::Duration::from_secs(5));
                std::process::exit(0x4);
            }

            println!("Done.");
            unsafe { CloseHandle(handle) };
        } else {
            println!("Couldn't open handle to process");
            std::thread::sleep(std::time::Duration::from_secs(5));
            std::process::exit(0x2);
        }
    } else {
        println!("Couldn't find Minecraft process, you sure it's running?");
        std::thread::sleep(std::time::Duration::from_secs(5));
        std::process::exit(0x1);
    }
}
