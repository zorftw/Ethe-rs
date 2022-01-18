use std::{collections::HashMap, fmt::Error, sync::Mutex};

use winapi::shared::{d3d9types::D3DCOLOR_ARGB, minwindef::MAX_PATH};

use crate::api::{processes, sdk::{*, self, activerenderinfo::world_to_screen, entity::{Vec2, Vec3}, minecraft::find_class}};

lazy_static::lazy_static! {
    pub static ref CLASSES: Mutex<HashMap<String, JClass>> = Mutex::new(HashMap::new());
}

pub fn spawn_instance(
    dictionary: sdk::JVMDictionary,
    handle: processes::NativeHandle,
) -> Option<Error> {
    {
        *CLASSES.lock().unwrap() = collect_all_classes(&dictionary, &handle);


        let minecraft_class = find_class("bao");
        let minecraft_object = minecraft::Minecraft::new(&minecraft_class, &handle);

        let world = minecraft_object.get_world(&handle);

        let mc_window =
            win_overlay::utils::find_window(Some(win_overlay::native_str!("LWJGL")), None)
                .expect("Couldn't find target window.");
        let overlay = win_overlay::Overlay::create_overlay(mc_window);

        let render_info = activerenderinfo::RenderInfo::new();

        let model_view_buffer = render_info.get_modelview(&handle);
        let projection_buffer = render_info.get_projection(&handle);
        let viewport_buffer = render_info.get_viewport(&handle);


        overlay.draw(&|| {

             let model_view = model_view_buffer.as_vec(&handle);
             let projection = projection_buffer.as_vec(&handle);
             let viewport = viewport_buffer.as_vec(&handle);

             let render_position = render_info.get_render_position(&handle);

             let players = world.get_players(&handle);

            players.iter().for_each(|player| {

                let mut feet_position: Vec2 = unsafe {core::mem::zeroed()};
                let mut head_position: Vec2 = unsafe { core::mem::zeroed()};

                let player_position = player.get_position(&handle);
                let last_tick_position = player.get_last_tick_position(&handle);
                let last_tick_head = Vec3 { x: last_tick_position.x, y: last_tick_position.y + 1.8f64, z: last_tick_position.z};

                let player_head_position = player.get_head_position(&handle);

                let x_feet = (last_tick_position.x + (player_position.x - last_tick_position.x) * 1.0f64) - render_position.x;
                let y_feet = (last_tick_position.y + (player_position.y - last_tick_position.y) * 1.0f64) - render_position.y;
                let z_feet = (last_tick_position.z + (player_position.z - last_tick_position.z) * 1.0f64) - render_position.z;

                let x_head = (last_tick_head.x + (player_head_position.x - last_tick_head.x) * 1.0f64) - render_position.x;
                let y_head = (last_tick_head.y + (player_head_position.y - last_tick_head.y) * 1.0f64) - render_position.y;
                let z_head = (last_tick_head.z + (player_head_position.z - last_tick_head.z) * 1.0f64) - render_position.z;

                let feet_pos = Vec3 { x: x_feet, y: y_feet, z: z_feet};
                let head_pos = Vec3 { x: x_head, y: y_head, z: z_head};

                if world_to_screen(
                    feet_pos,
                    &mut feet_position,
                    &model_view,
                    &projection,
                    &viewport,
                ) && world_to_screen(
                    head_pos,
                    &mut head_position,
                    &model_view,
                    &projection,
                    &viewport,
                ) {

                    if feet_position.x >= 0f64
                        && feet_position.y >= 0f64
                        && head_position.x >= 0f64
                        && head_position.y >= 0f64
                    {

                        let width = (feet_position.y - head_position.y) / 3f64;

                        overlay.draw_box(
                            (feet_position.x - (width / 2f64)) as _,
                            head_position.y as _,
                            (width * 2f64) as _,
                            (feet_position.y - head_position.y) as _,
                            1,
                            D3DCOLOR_ARGB(255, 255, 0, 0),
                        );
                    }
                }
            });
            std::thread::sleep(std::time::Duration::from_millis(1));
        });
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
            let mut entry_native = processes::read_class_original::<DictionaryEntry>(
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
                    processes::read_class_original::<DictionaryEntry>(handle, entry.read().next() as _);
                entry = entry_native.get() as *mut DictionaryEntry;

                classes.push(entry.read());
            }
        })
    }

    classes.into_iter()
}
