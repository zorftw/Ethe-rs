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
    next: usize,
    pub klass: *mut usize,
    pub loader: usize,
}

impl DictionaryEntry {
    pub fn next(&self) -> usize {
        self.next & 0xFFFFFFFFFFFFFFFE
    }
}
