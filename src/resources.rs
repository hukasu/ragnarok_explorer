use bevy::{asset::Handle, prelude::Resource, text::Font};

use ragnarok_rebuild_bevy::assets::grf::GRF;

#[derive(Debug, Resource)]
pub struct LoadingFont(pub Handle<Font>);

#[derive(Resource)]
pub struct OpenGrf {
    pub filename: String,
    pub grf: GRF,
}
