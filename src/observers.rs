use std::{collections::HashMap, path::PathBuf};

use bevy::{
    core::Name,
    prelude::{BuildChildren, Commands, Entity, Trigger},
};

use ragnarok_rebuild_bevy::assets::grf::GRF;

use crate::{
    components::{FileTreeNode, FileTreeRoot},
    events::LoadGrf,
    resources::OpenGrf,
};

pub fn open_grf(trigger: Trigger<LoadGrf>, mut commands: Commands) {
    let Ok(grf) = GRF::new(trigger.event()).inspect_err(|err| bevy::log::error!("{}", err)) else {
        return;
    };

    let filename = trigger
        .event()
        .file_name()
        .and_then(|filename| filename.to_str())
        .unwrap_or("Unnamed");

    let mut file_entity_map = HashMap::new();
    file_entity_map.insert(
        PathBuf::from("data"),
        commands
            .spawn((Name::new("data"), FileTreeRoot, FileTreeNode))
            .id(),
    );
    for grf_entry_path in grf.iter_filenames().cloned() {
        let filename = grf_entry_path
            .file_name()
            .and_then(|filename| filename.to_str())
            .unwrap_or("Unnamed");

        let tree_entry = commands.spawn(Name::new(filename.to_owned())).id();

        add_node_to_parent(
            &mut commands,
            tree_entry,
            &mut file_entity_map,
            grf_entry_path,
        )
    }

    commands.insert_resource(OpenGrf {
        filename: filename.to_owned(),
        grf,
    });
}

fn add_node_to_parent(
    commands: &mut Commands,
    entity: Entity,
    file_entity_map: &mut HashMap<PathBuf, Entity>,
    mut path: PathBuf,
) {
    path.pop();
    match file_entity_map.entry(path.clone()) {
        std::collections::hash_map::Entry::Vacant(entry) => {
            let filename = path
                .file_name()
                .and_then(|filename| filename.to_str())
                .unwrap_or("Unnamed");
            let node = commands
                .spawn((Name::new(filename.to_owned()), FileTreeNode))
                .add_child(entity)
                .id();
            entry.insert(node);
            add_node_to_parent(commands, node, file_entity_map, path)
        }
        std::collections::hash_map::Entry::Occupied(entry) => {
            commands.entity(*entry.get()).add_child(entity);
        }
    }
}
