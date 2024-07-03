use crate::{
    assets::ImageAssets,
    kenney_assets::KenneySpriteSheetAsset,
    ship::{PlayerEngineFire, PlayerShipType},
    ui::pause::Pausable,
    GameState, Player,
};
use bevy::prelude::*;
use bevy_xpbd_2d::prelude::*;
use std::time::Duration;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MovementFactor>().add_systems(
            Update,
            (
                player_movement_system
                    .run_if(in_state(GameState::Playing)),
                weapon_system
                    .run_if(in_state(GameState::Playing)),
                engine_fire
                    .run_if(in_state(GameState::Playing)),
                laser_movement,
            )
                .run_if(resource_equals(
                    Pausable::NotPaused,
                )),
        );
    }
}

#[derive(Component)]
pub struct Laser{
    /// movement factor is ship's movement speed at time of firing
pub    movement_factor: Vec2,
/// speed is laser's inherent movement speed
pub speed: f32,
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct MovementFactor(pub Vec2);

fn laser_movement(
    mut lasers: Query<(&mut Transform, &Laser)>,
    time: Res<Time>,
) {
    for (
        mut transform,
        Laser{
            movement_factor,
            speed
        },
    ) in &mut lasers
    {
        let laser_facing_direction =
            transform.rotation * Vec3::Y;
        let translation_delta = *movement_factor
            + laser_facing_direction.xy()
                * *speed
                * time.delta_seconds();
        transform.translation.x += translation_delta.x;
        transform.translation.y += translation_delta.y;
    }
}

#[derive(Component)]
pub struct PlayerOwned;

fn weapon_system(
    mut commands: Commands,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<&Transform, With<Player>>,
    movement_factor: ResMut<MovementFactor>,
    images: Res<ImageAssets>,
    sheets: Res<Assets<KenneySpriteSheetAsset>>,
    mut last_shot: Local<Option<Duration>>,
) {
    let space_sheet =
        sheets.get(&images.space_sheet).unwrap();

    let Ok(transform) = query.get_single() else {
        if query.iter().count() > 1 {
            error_once!(
                "Only expected one Player component. got {}",
                query.iter().count()
            );
        }
        return;
    };

    if keyboard_input.pressed(KeyCode::Space) {
        let can_shoot = last_shot.is_none() || {
            if let Some(shot) = *last_shot {
                time.elapsed() - shot
                    > Duration::from_millis(200)
            } else {
                false
            }
        };

        if can_shoot {
            *last_shot = Some(time.elapsed());

            commands.spawn((
                SpriteBundle {
                    transform: *transform,
                    texture: space_sheet.sheet.clone(),
                    ..default()
                },
                TextureAtlas {
                    layout: space_sheet
                        .texture_atlas_layout
                        .clone(),
                    index: 105,
                },
                Laser{
                    movement_factor: **movement_factor,
                    speed: 1000.
                },
                PlayerOwned,
                Collider::triangle(
                    Vec2::new(0., -27.),
                    Vec2::new(4.5, 27.),
                    Vec2::new(-4.5, 27.),
                ),
            ));
        }
    }
}

fn engine_fire(
    mut query: Query<
        &mut Visibility,
        With<PlayerEngineFire>,
    >,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        for mut visibility in query.iter_mut() {
            *visibility = Visibility::Visible;
        }
    } else {
        for mut visibility in query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}
fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (&mut Transform, &PlayerShipType),
        With<Player>,
    >,
    mut movement_factor: ResMut<MovementFactor>,
) {
    let Ok((mut transform, ship)) = query.get_single_mut()
    else {
        if query.iter().count() > 1 {
            error_once!(
                "Expected zero or one Player component. got {}",
                query.iter().count()    
            );
        }
        return;
    };

    let mut rotation_factor = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        rotation_factor += 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        rotation_factor -= 1.0;
    }

    // update the ship rotation around the Z axis
    // (perpendicular to the 2D plane of the screen)
    transform.rotate_z(
        rotation_factor
            * ship.base_ship_speed().rotation_speed
            * time.delta_seconds(),
    );

    // get the ship's forward vector by applying the
    // current rotation to the ships initial facing
    // vector
    let user_facing_direction =
        transform.rotation * Vec3::Y;

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        movement_factor.0 = (movement_factor.0
            + 0.01 * user_facing_direction.xy())
        .clamp(Vec2::splat(-1.0), Vec2::splat(1.0));
    } else {
        // decay movement factor?
        // movement_factor.0 = (movement_factor.0
        // * 0.95) .clamp(Vec2::splat(-1.0),
        // Vec2::splat(1.0));
    }
    // get the distance the ship will move based on
    // direction, the ship's movement speed and delta
    // time
    let movement_distance = movement_factor.0
        * ship.base_ship_speed().movement_speed
        * time.delta_seconds();
    // create the change in translation using the new
    // movement direction and distance
    let translation_delta = movement_distance;
    // update the ship translation with our new
    // translation delta
    transform.translation.x += translation_delta.x;
    transform.translation.y += translation_delta.y;
}
