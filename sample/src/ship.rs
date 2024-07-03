use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_xpbd_2d::prelude::*;

use crate::{
    assets::ImageAssets,
    controls::MovementFactor,
    kenney_assets::KenneySpriteSheetAsset,
    lives::{Lives, RemoveLifeEvent},
    movement::WrappingMovement,
    ui::pause::Pausable,
    GameState, Player,
};

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerShipType::B)
            .add_systems(
                PostUpdate,
                player_ship_destroyed_event_handler
                    .run_if(resource_equals(
                        Pausable::NotPaused,
                    ))
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                spawn_ship_after_ship_destroyed
                    .run_if(in_state(GameState::Playing)),
                // if lives have changed and is not 0
                // .run_if(resource_changed::<Lives>)
                // .run_if(not(resource_equals(Lives(0))))
                // if ship was just destroyed
                // .run_if(on_event::<RemoveLifeEvent>()),
            )
            .add_event::<ShipDestroyed>();
    }
}

#[derive(Component)]
pub struct PlayerEngineFire;

#[derive(Event)]
pub struct ShipDestroyed {
    pub destroyed_at: Transform,
    pub ship_type: PlayerShipType,
}

#[derive(Bundle)]
pub struct ShipBundle {
    pub sprite_bundle: SpriteBundle,
    pub texture_atlas: TextureAtlas,
    pub player: Player,
    pub ship_type: PlayerShipType,
    pub collider: Collider,
    pub wrapping_movement: WrappingMovement,
}

#[derive(Resource, Component, Clone)]
pub enum PlayerShipType {
    A,
    B,
    C,
}

impl PlayerShipType {
    pub fn base_atlas_index(&self) -> usize {
        match &self {
            PlayerShipType::A => 200,
            PlayerShipType::B => 207,
            PlayerShipType::C => 214,
        }
    }
    pub fn life_atlas_index(&self) -> usize {
        match &self {
            PlayerShipType::A => 188,
            PlayerShipType::B => 192,
            PlayerShipType::C => 196,
        }
    }
    pub fn all_ships() -> Vec<PlayerShipType> {
        vec![
            PlayerShipType::A,
            PlayerShipType::B,
            PlayerShipType::C,
        ]
    }
    pub fn collider(&self) -> Collider {
        Collider::capsule(40., 10.)
    }
    pub fn base_ship_speed(&self) -> BaseShipSpeed {
        match self {
            PlayerShipType::A => BaseShipSpeed {
                movement_speed: 500.0, // meters per second
                rotation_speed: f32::to_radians(360.0), /* degrees per second */
            },
            PlayerShipType::B => BaseShipSpeed {
                movement_speed: 500.0, // meters per second
                rotation_speed: f32::to_radians(360.0), /* degrees per second */
            },
            PlayerShipType::C => BaseShipSpeed {
                movement_speed: 500.0, // meters per second
                rotation_speed: f32::to_radians(360.0), /* degrees per second */
            },
        }
    }
}

pub struct BaseShipSpeed {
    /// linear speed in meters per second
    pub movement_speed: f32,
    /// rotation speed in radians per second
    pub rotation_speed: f32,
}

fn player_ship_destroyed_event_handler(
    mut commands: Commands,
    images: Res<ImageAssets>,
    mut events: EventReader<ShipDestroyed>,
    sheets: Res<Assets<KenneySpriteSheetAsset>>,
    mut effect: Query<(
        &mut EffectProperties,
        &mut EffectSpawner,
        &mut Transform,
    )>,
    mut ship_movement: ResMut<MovementFactor>,
    mut life_events: EventWriter<RemoveLifeEvent>,
) {
    let Some(space_sheet) = sheets.get(&images.space_sheet)
    else {
        warn!("player_ship_destroyed_event_handler requires meteor sprites to be loaded");
        return;
    };

    let Ok((
        mut properties,
        mut spawner,
        mut effect_transform,
    )) = effect.get_single_mut()
    else {
        warn!("effect not ready yet, returning");
        return;
    };

    for ShipDestroyed {
        destroyed_at,
        ship_type,
    } in &mut events.read()
    {
        effect_transform.translation =
            destroyed_at.translation;

        let color = Color::lch(
            1.,
            1.,
            rand::random::<f32>() * 360.,
        )
        .as_linear_rgba_u32();
        properties.set("spawn_color", color.into());

        // Spawn the particles
        spawner.reset();

        ship_movement.0 = Vec2::ZERO;

        life_events.send(RemoveLifeEvent);
    }
}

fn spawn_ship_after_ship_destroyed(
    mut commands: Commands,
    images: Res<ImageAssets>,
    sheets: Res<Assets<KenneySpriteSheetAsset>>,
    lives: Res<Lives>,
    player_ship_type: Res<PlayerShipType>,
) {
    if !lives.is_changed() || lives.0 == 0 || lives.0 == 3 {
        return;
    }
    let Some(space_sheet) = sheets.get(&images.space_sheet)
    else {
        warn!("player_ship_destroyed_event_handler requires meteor sprites to be loaded");
        return;
    };

    let engine_fire = commands
        .spawn((
            SpriteBundle {
                transform: Transform::from_xyz(
                    0., -60., 1.,
                ),
                texture: space_sheet.sheet.clone(),
                sprite: Sprite {
                    flip_y: true,
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            TextureAtlas {
                index: 74,
                layout: space_sheet
                    .texture_atlas_layout
                    .clone(),
            },
            PlayerEngineFire,
        ))
        .id();

    commands
        .spawn(ShipBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform::from_xyz(0., 0., 1.),
                texture: space_sheet.sheet.clone(),
                ..default()
            },
            texture_atlas: TextureAtlas {
                index: player_ship_type.base_atlas_index(),
                layout: space_sheet
                    .texture_atlas_layout
                    .clone(),
            },
            player: Player,
            ship_type: player_ship_type.clone(),
            collider: player_ship_type.collider(),
            wrapping_movement: WrappingMovement,
        })
        .add_child(engine_fire);
}
