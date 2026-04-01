use lupabase::prelude::*;
use std::{path::PathBuf, sync::LazyLock};

pub static CURRENT_DIR: LazyLock<PathBuf> = LazyLock::new(|| std::env::current_dir().unwrap());
pub static CURRENT_FILES_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| std::path::absolute(CURRENT_DIR.join("files/")).unwrap());

pub static DATABASE: LazyLock<DiskDB<CborSerde>> =
    LazyLock::new(|| DiskDB::new(&*CURRENT_FILES_DIR));
