use crate::{
    config::NodeId,
    node_atlas::{LoadNodeEvent, NodeAtlas, NodeAttachment},
};
use bevy::{
    asset::{AssetServer, HandleId, LoadState},
    prelude::*,
    render::render_resource::*,
    utils::HashMap,
};

pub struct TextureAttachmentFromDisk {
    pub path: String,
    pub texture_descriptor: TextureDescriptor<'static>,
}

#[derive(Default, Component)]
pub struct TextureAttachmentFromDiskLoader {
    pub attachments: HashMap<String, TextureAttachmentFromDisk>,
    /// Maps the id of an asset to the corresponding node id.
    pub handle_mapping: HashMap<HandleId, (NodeId, String)>,
}

impl TextureAttachmentFromDiskLoader {
    pub fn add_attachment(&mut self, label: String, attachment: TextureAttachmentFromDisk) {
        self.attachments.insert(label, attachment);
    }
}

pub fn start_loading_attachment_from_disk(
    mut load_events: EventReader<LoadNodeEvent>,
    asset_server: Res<AssetServer>,
    mut terrain_query: Query<(&mut NodeAtlas, &mut TextureAttachmentFromDiskLoader)>,
) {
    for (mut node_atlas, mut config) in terrain_query.iter_mut() {
        let TextureAttachmentFromDiskLoader {
            ref mut attachments,
            ref mut handle_mapping,
        } = config.as_mut();

        for &LoadNodeEvent(node_id) in load_events.iter() {
            let node = node_atlas.loading_nodes.get_mut(&node_id).unwrap();

            for (label, TextureAttachmentFromDisk { ref path, .. }) in attachments.iter() {
                let handle: Handle<Image> = asset_server.load(&format!("{path}/{node_id}.png"));

                if asset_server.get_load_state(handle.clone()) == LoadState::Loaded {
                    node.loaded(label);
                } else {
                    handle_mapping.insert(handle.id, (node_id, label.clone()));
                };

                node.set_attachment(label.clone(), NodeAttachment::Texture { handle });
            }
        }
    }
}

pub fn finish_loading_attachment_from_disk(
    mut asset_events: EventReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
    mut terrain_query: Query<(&mut NodeAtlas, &mut TextureAttachmentFromDiskLoader)>,
) {
    for event in asset_events.iter() {
        if let AssetEvent::Created { handle } = event {
            for (mut node_atlas, mut config) in terrain_query.iter_mut() {
                if let Some((node_id, label)) = config.handle_mapping.remove(&handle.id) {
                    let image = images.get_mut(handle).unwrap();
                    let attachment = config.attachments.get(&label).unwrap();

                    image.texture_descriptor = attachment.texture_descriptor.clone();

                    let node = node_atlas.loading_nodes.get_mut(&node_id).unwrap();
                    node.loaded(&label);
                    break;
                }
            }
        }
    }
}