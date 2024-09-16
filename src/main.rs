mod components;
mod events;
mod observers;
mod resources;

use std::ops::Deref;

use bevy::{
    app::{App, Startup, Update},
    asset::{AssetServer, Assets},
    core::Name,
    ecs::system::QueryLens,
    prelude::{
        resource_exists, Camera3dBundle, Children, Commands, Entity, Has, IntoSystemConfigs, Query,
        Res, With,
    },
    text::Font,
    DefaultPlugins,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use self::{
    components::{FileTreeNode, FileTreeRoot},
    events::LoadGrf,
    resources::{LoadingFont, OpenGrf},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
        .add_systems(Startup, (spawn_camera, init_font_loading))
        .add_systems(
            Update,
            check_loading_font.run_if(resource_exists::<LoadingFont>),
        )
        .add_systems(Update, ui_example_system)
        .observe(observers::open_grf)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        ..Default::default()
    });
}

fn init_font_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("NotoSansCJK-VF.otf.ttc");
    commands.insert_resource(LoadingFont(font));
}

fn check_loading_font(
    mut commands: Commands,
    mut contexts: EguiContexts,
    loading_font: Res<LoadingFont>,
    fonts: Res<Assets<Font>>,
) {
    let Some(font) = fonts.get(&loading_font.0) else {
        return;
    };

    commands.remove_resource::<LoadingFont>();

    let font_name = "NotoSansCJK-VF".to_owned();

    let font_box: &dyn ab_glyph::Font = &font.font;
    let font_data = egui::FontData::from_owned(font_box.font_data().to_vec());
    let mut font_definitons = egui::FontDefinitions::default();
    font_definitons
        .font_data
        .insert(font_name.clone(), font_data);

    let font_family = egui::FontFamily::Proportional;
    let Some(font_family_store) = font_definitons.families.get_mut(&font_family) else {
        return;
    };
    font_family_store.insert(0, font_name);

    contexts.ctx_mut().set_fonts(font_definitons);
}

fn ui_example_system(
    mut contexts: EguiContexts,
    mut commands: Commands,
    open_grf: Option<Res<OpenGrf>>,
    file_tree: Query<Entity, With<FileTreeRoot>>,
    mut children: Query<&Children>,
    mut nodes: Query<(&Name, Has<FileTreeNode>)>,
) {
    egui::SidePanel::left("file_list")
        .width_range(egui::Rangef::new(200., 600.))
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Ragnarok Explorer");

            let open_grf_ref = open_grf.as_ref();
            ui.horizontal(|ui| {
                let open_file = open_grf_ref
                    .map(|res| res.filename.clone())
                    .unwrap_or("No file open".to_owned());

                if ui.button("Open Grf").clicked() {
                    // TODO proper file dialog
                    commands.trigger(LoadGrf {
                        path: "data.grf".into(),
                    });
                };
                ui.label(open_file);
            });

            ui.separator();

            if open_grf.is_some() {
                let Ok(file_tree_root) = file_tree.get_single() else {
                    return;
                };
                egui::ScrollArea::new(egui::Vec2b::new(false, true))
                    .auto_shrink(egui::Vec2b::new(false, false))
                    .show(ui, |ui| {
                        grf_file_tree(
                            ui,
                            file_tree_root,
                            children.transmute_lens(),
                            nodes.transmute_lens(),
                        );
                    });
            }
        });
}

fn grf_file_tree(
    ui: &mut egui::Ui,
    file_tree_root: Entity,
    mut children: QueryLens<&Children>,
    mut nodes: QueryLens<(&Name, Has<FileTreeNode>)>,
) {
    let children_query = children.query();
    let nodes_query = nodes.query();

    let Ok((name, is_tree_node)) = nodes_query.get(file_tree_root) else {
        return;
    };
    let node_children = match children_query.get(file_tree_root) {
        Ok(children) => children.deref(),
        Err(_) => &[],
    }
    .to_vec();

    if is_tree_node {
        ui.collapsing(name.to_string(), |sub_items| {
            let mut children_query = children_query;
            let mut nodes_query = nodes_query;
            for child in node_children {
                grf_file_tree(
                    sub_items,
                    child,
                    children_query.transmute_lens(),
                    nodes_query.transmute_lens(),
                );
            }
        });
    } else {
        ui.label(name.to_string());
    }
}
