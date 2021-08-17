use std::{collections::HashMap, fmt::Error};

use winapi::shared::minwindef::MAX_PATH;

use crate::api::{
    processes,
    sdk::{self, *},
};

pub fn spawn_instance(
    dictionary: sdk::JVMDictionary,
    handle: processes::NativeHandle,
) -> Option<Error> {
    {
        let collection = collect_all_classes(&dictionary, &handle);

        // let minecraft_class = &collection.get("bao").expect("Not found!");

        // minecraft_class.iterate_fields(&handle).for_each(|entry| {
        //     println!("Field: {}({}) @ {:p}", entry.name, entry.sig, entry._field_info.offset() as *mut usize);
        // })

        println!("Minecraft:");
        let render_info = &collection.get("bao").expect("Not found!");
        render_info.dump_all_fields(&handle);

        println!("World:");
        let world = &collection.get("bjf").expect("Not found!");
        world.dump_all_fields(&handle);

        println!("Static fields: {:p}", render_info.static_fields);

        let minecraft_object = minecraft::Minecraft::new(render_info, &handle);
        println!("{:x}", minecraft_object._address);

        println!("World: {:x}", minecraft_object.get_world_pointer(&handle));
        // const VIEWPORT_OFFSET: usize = 0x68; // hardcoded here but can be easily found using function above
        // const MODEL_VIEW_OFFSET: usize = 0x6c;
        // const PROJECTION_OFFSET: usize = 0x70;
        // let viewport_address = render_info.static_fields as usize + VIEWPORT_OFFSET;
        // let modelview_address = render_info.static_fields as usize + MODEL_VIEW_OFFSET;
        // let projection_address = render_info.static_fields as usize + PROJECTION_OFFSET;

        // println!("Static fields: {:p}", render_info.static_fields);
        // println!("Viewport: {:p} + Model view: {:p} + Projection: {:p}", viewport_address as *mut usize, modelview_address as *mut usize, projection_address as *mut usize);
    
        // let mut viewport_pointer: u32 = 0;
        // processes::read(&handle, viewport_address, &mut viewport_pointer);

        // println!("Viewport pointer: {:x}", viewport_pointer as usize);

        // let viewport = java::JavaBuffer::from_native(&handle, viewport_pointer as *mut java::JavaBuffer<i32>);

        // println!("Values: {} + {}", viewport.get(&handle, 2).expect("lol"), viewport.get(&handle, 3).expect("lol"));
    }

    None
}

pub fn collect_all_classes(
    dictionary: &sdk::JVMDictionary,
    handle: &processes::NativeHandle,
) -> HashMap<String, JClass> {
    let mut classes: HashMap<String, JClass> = HashMap::new();

    for entry in iterate_classes(&dictionary, &handle) {
        let clazz = JClass::from_native(&handle, entry.klass);

        if clazz.symbol != std::ptr::null_mut() {
            let symbol = JSymbol::from_native(&handle, clazz.symbol);

            // maybe its because were external, but the JVM seems to leave empty or disposed off pointers dangling, leading
            // us to the most retarded text you can find, not ideal, so lets limit the length of classnames to MAX_PATH like the god
            // bill gates intended
            if symbol.lenght < MAX_PATH as _ {
                let name = symbol.to_string(&handle);

                if name.eq("bao") {
                    println!("Minecraft: {:p}", entry.klass);
                } else if name.contains("baj") {
                    println!("{}", name);
                }

                classes.insert(symbol.to_string(&handle), clazz);
            }
        }
    }

    classes
}

pub fn iterate_classes(
    dictionary: &sdk::JVMDictionary,
    handle: &processes::NativeHandle,
) -> impl Iterator<Item = sdk::DictionaryEntry> {
    let mut classes: Vec<sdk::DictionaryEntry> = Vec::new();

    unsafe {
        (0..dictionary.table_size).for_each(|idx| {
            let mut entry_native = processes::read_class::<DictionaryEntry>(
                handle,
                dictionary.entries.offset(idx as _) as _,
            );

            let mut entry = entry_native.get() as *mut DictionaryEntry;

            // push back first class
            classes.push(entry.read());

            while entry != std::ptr::null_mut() {
                if entry.read().next == 0 || entry.read().klass == std::ptr::null_mut() {
                    break;
                }

                entry_native =
                    processes::read_class::<DictionaryEntry>(handle, entry.read().next() as _);
                entry = entry_native.get() as *mut DictionaryEntry;

                classes.push(entry.read());
            }
        })
    }

    classes.into_iter()
}
