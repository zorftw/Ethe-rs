use std::sync::Mutex;

use crate::api::{
    processes::{self, NativeHandle},
    sdk::minecraft::find_class,
};

use super::{
    entity::Entity,
    java::{self, JavaArray},
    FromNative, JClass,
};

#[derive(Debug, Default)]
pub struct World {
    _clazz: JClass,
    pub _address: usize,
}

lazy_static::lazy_static! {
    static ref PLAYERS_POINTERS_OFFSET: Mutex<usize> = Mutex::new(0usize);
}

impl World {
    pub fn new(class: &JClass, address: u32) -> Self {
        Self {
            _clazz: class.clone(),
            _address: address as _,
        }
    }

    pub fn get_players(&self, handle: &NativeHandle) -> Vec<Entity> {
        let mut res = vec![];

        let players = self.get_players_pointers(handle);

        for i in 0i32..players.length {
            let address = players.get_at(handle, i);

            res.push(Entity::new(
                address.expect("Couldn't get address of player"),
            ));
        }

        drop(players);

        res
    }

    #[allow(unused)]
    pub fn get_players_pointers(&self, handle: &NativeHandle) -> JavaArray<u32> {
        if *PLAYERS_POINTERS_OFFSET.lock().unwrap() == 0usize {
            let _clazz = find_class("bjf");
            *PLAYERS_POINTERS_OFFSET.lock().unwrap() = _clazz
                .find_field_entry(&handle, "h", "Ljava/util/List;")
                .expect("Couldn't find playerEntities field")
                ._field_info
                .offset() as usize;
        }

        let mut player_entities_pointer: u32 = 0;

        processes::read(
            &handle,
            (self._address + *PLAYERS_POINTERS_OFFSET.lock().unwrap()) as usize,
            &mut player_entities_pointer,
        );

        java::JavaArray::from_native(
            &handle,
            player_entities_pointer as *mut java::JavaArray<u32>,
        )
    }
}
