use std::{i32, str};

use winapi::{
    shared::minwindef::DWORD,
    um::{
        memoryapi::{ReadProcessMemory, VirtualProtectEx, VirtualQueryEx},
        sysinfoapi::{GetSystemInfo, SYSTEM_INFO},
        winnt::{
            HANDLE, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_EXECUTE_READWRITE, PAGE_NOACCESS,
        },
    },
};

use super::processes::ModuleEntry;

pub struct Signature {
    sig: String,
}

/// Example signature: "AA BB CC DD ?? CC ?? DD ?? 00"
impl Signature {
    /// Create a new signature
    pub fn new(sig: &str) -> Self {
        Self {
            sig: sig.to_string(),
        }
    }

    /// Convert the signature to a byte array (vector)
    pub fn to_bytes(&self) -> Vec<i32> {
        self.sig
            .replace(' ', "")
            .as_bytes()
            .chunks(2)
            .map(str::from_utf8)
            .collect::<Result<Vec<&str>, _>>()
            .unwrap()
            .into_iter()
            .map(|q| {
                if !q.contains("?") {
                    i32::from_str_radix(q, 16).unwrap()
                } else {
                    0xCC
                }
            })
            .collect()
    }
}

pub fn pattern_scan_module(
    handle: HANDLE,
    pattern: &Signature,
    module: ModuleEntry,
) -> Option<usize> {
    // Check if the pattern is valid
    if pattern.to_bytes().len() == 0 {
        println!("Invalid pattern.");
        return None;
    }

    unsafe {
        // Spawn buffer for module
        let mut module_buffer = vec![0 as i8; module.size];
        let mut old_protection: DWORD = 0;

        let pattern_to_bytes = pattern.to_bytes();

        // make sure we can access it
        VirtualProtectEx(
            handle,
            module.base as _,
            module.size,
            PAGE_EXECUTE_READWRITE,
            &mut old_protection as *mut _,
        );

        // Read the module
        if ReadProcessMemory(
            handle,
            module.base as _,
            module_buffer.as_mut_ptr() as *mut _,
            module.size,
            std::ptr::null_mut(),
        ) != 1
        {
            println!(
                "Unable to read module {} @ {:p} with size {}",
                module.name, module.base as *mut i8, module.size
            );
        }

        let search = || {
            for i in 0..(module.size - pattern_to_bytes.len()) {
                let mut found = true;

                for j in 0..(pattern_to_bytes.len()) {
                    if module_buffer[i + j] != pattern_to_bytes[j] as _
                        && pattern_to_bytes[j] != 0xCC
                    {
                        found = false;
                        break;
                    }
                }

                if found {
                    return Some(module.base as usize + i);
                }
            }
            None
        };

        // search
        if let Some(address) = search() {
            // reset page protection
            VirtualProtectEx(
                handle,
                module.base as _,
                module.size,
                old_protection,
                std::ptr::null_mut(),
            );

            // return address
            return Some(address);
        } else {
            // Do nothing
        }

        // reset old protection
        VirtualProtectEx(
            handle,
            module.base as _,
            module.size,
            old_protection,
            std::ptr::null_mut(),
        );
    }

    None
}

#[allow(dead_code)]
pub fn pattern_scan_memory(handle: HANDLE, pattern: &Signature) -> Option<usize> {
    if pattern.to_bytes().len() == 0 {
        println!("Invalid pattern.");
        return None;
    }

    unsafe {
        // allocate buffer for SYSTEM_INFO structure
        let mut system_info_buffer = [0 as i8; std::mem::size_of::<SYSTEM_INFO>()];
        GetSystemInfo(system_info_buffer.as_mut_ptr() as *mut SYSTEM_INFO);

        // change the type
        let system_info: SYSTEM_INFO = (system_info_buffer.as_mut_ptr() as *mut SYSTEM_INFO).read();

        let end_of_space = system_info.lpMaximumApplicationAddress as usize;

        let pattern_to_bytes = pattern.to_bytes();

        let mut chunk: usize = 0;

        // allocate buffer with size of region
        let mut memory_buffer = vec![0 as i8; 0];

        while chunk < end_of_space {
            // allocate buffer for MEMORY_BASIC_INFORMATION buffer
            let mut mbi_buffer = [0 as i8; std::mem::size_of::<MEMORY_BASIC_INFORMATION>()];

            // query memory region
            if !VirtualQueryEx(
                handle,
                chunk as _,
                mbi_buffer.as_mut_ptr() as *mut MEMORY_BASIC_INFORMATION,
                std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            ) == 0
            {
                return None;
            }

            // change buffer to actual struct
            let mbi = (mbi_buffer.as_mut_ptr() as *mut MEMORY_BASIC_INFORMATION).read();

            // can we access the region?
            if mbi.State == MEM_COMMIT && mbi.Protect != PAGE_NOACCESS {
                let mut old_protection: DWORD = 0;

                // resize buffer
                memory_buffer.resize(mbi.RegionSize, 0);

                // make sure we can read the page
                if VirtualProtectEx(
                    handle,
                    mbi.BaseAddress,
                    mbi.RegionSize,
                    PAGE_EXECUTE_READWRITE,
                    &mut old_protection,
                ) != 0
                {
                    // copy the region to our local memory
                    if ReadProcessMemory(
                        handle,
                        mbi.BaseAddress,
                        memory_buffer.as_mut_ptr() as _,
                        mbi.RegionSize,
                        std::ptr::null_mut(),
                    ) == 0
                    {
                        println!(
                            "Unable to RPM region: {:p} with size of {}",
                            mbi.BaseAddress, mbi.RegionSize
                        );
                        continue;
                    }

                    // based lambdas
                    let search = || {
                        for i in 0..(mbi.RegionSize - pattern_to_bytes.len()) {
                            let mut found = true;

                            for j in 0..pattern_to_bytes.len() {
                                if memory_buffer[i + j] != pattern_to_bytes[j] as _
                                    && pattern_to_bytes[j] != 0xCC
                                {
                                    found = false;
                                    break;
                                }
                            }

                            if found {
                                return Some(mbi.BaseAddress as usize + i);
                            }
                        }
                        None
                    };

                    // search
                    if let Some(address) = search() {
                        // reset page protection
                        VirtualProtectEx(
                            handle,
                            mbi.BaseAddress,
                            mbi.RegionSize,
                            old_protection,
                            std::ptr::null_mut(),
                        );

                        // return address
                        return Some(address);
                    } else {
                        // Do nothing
                    }

                    // reset page protection
                    VirtualProtectEx(
                        handle,
                        mbi.BaseAddress,
                        mbi.RegionSize,
                        old_protection,
                        std::ptr::null_mut(),
                    );
                }
            }

            chunk += mbi.RegionSize;
        }
    }

    println!("Done.");

    None
}
