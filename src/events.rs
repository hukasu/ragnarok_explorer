use std::path::PathBuf;

use bevy::prelude::{Deref, Event};

#[derive(Debug, Event, Deref)]
pub struct LoadGrf {
    pub path: PathBuf,
}
