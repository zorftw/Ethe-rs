use std::{collections::HashMap, fmt::Error, hash::Hash};

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

        let minecraft_class = &collection.get("bao").expect("Not found!");

        minecraft_class.iterate_fields(&handle).for_each(|entry| {
            println!("      Field -> Name({}) + Sig({})", entry.field_info.name_idx(), entry.field_info.sig_idx());
        })
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
                let name = symbol.to_string(clazz.symbol as usize, &handle);

                if name.eq("bao") {
                    println!("Minecraft: {:p}", entry.klass);
                }

                classes.insert(symbol.to_string(clazz.symbol as usize, &handle), clazz);
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
