use bevy::{math::Vec3Swizzles, prelude::*, render::camera::OrthographicProjection};

use crate::MainCamera;

pub struct ClickablePlugin;
impl Plugin for ClickablePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new()
                .label(ClickExecutionLabel::Main)
                .with_system(
                    remove_clicked_system
                        .system()
                        .label(ClickExecutionLabel::RemoveClicked),
                )
                .with_system(hover_2d_system.system().label(ClickExecutionLabel::Hover))
                .with_system(
                    press_down_system
                        .system()
                        .label(ClickExecutionLabel::PressDown)
                        .after(ClickExecutionLabel::RemoveClicked)
                        .after(ClickExecutionLabel::Hover),
                )
                .with_system(release_system.system().label(ClickExecutionLabel::Release))
                .with_system(
                    remove_selected_system
                        .system()
                        .label(ClickExecutionLabel::RemoveSelected),
                ),
        );
    }
}

//TODO Button Type
#[derive(Component)]
pub struct Clickable {
    pub size: Vec2,
    pub active: bool,
}

#[derive(Component)]
pub struct Hovered;

#[derive(Component)]
pub struct Selected;

///Click event element that stays until next klick, ore removed
#[derive(Component)]
pub struct Clicked;

#[derive(Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
enum ClickExecutionLabel {
    Main,
    Hover,
    RemoveClicked,
    PressDown,
    Release,
    RemoveSelected,
}

//FIXME Multiple Hovered components could be created
fn hover_2d_system(
    mut commands: Commands,
    query: Query<(
        Entity,
        &GlobalTransform,
        &Transform,
        &Clickable,
        Option<&Hovered>,
    )>,
    query_camera: Query<(&GlobalTransform, &OrthographicProjection), With<MainCamera>>,
    mut cursor_moved_event: EventReader<CursorMoved>,
) {
    if let Some(cursor_moved) = cursor_moved_event.iter().next() {
        if let Ok((camera_transform, orthographic_projection)) = query_camera.get_single() {
            let cursor_pos = cursor_moved.position
                + Vec2::new(orthographic_projection.left, orthographic_projection.bottom)
                + camera_transform.translation.xy();

            //FIXME Rotation and Scale not implemented
            //FIXME No blocking. Solution: Order query by z
            let mut element_found = false;
            let mut query_order = query.iter().collect::<Vec<_>>();
            query_order.sort_by(|(_, a_global, a_local, ..), (_, b_global, b_local, ..)| {
                let a = a_global.translation.z + a_local.translation.z;
                let b = b_global.translation.z + b_local.translation.z;
                a.partial_cmp(&b).unwrap()
            });
            for (entity, global_transform, _, clickable, o_hovered) in query_order.into_iter() {
                let (x, y, _): (f32, f32, _) = global_transform.translation.into();
                let (width, height) = clickable.size.into();
                let rect = (
                    x - width / 2.0,
                    y - height / 2.0,
                    x + width / 2.0,
                    y + height / 2.0,
                );

                if !element_found
                    && clickable.active
                    && cursor_pos.x >= rect.0
                    && cursor_pos.x <= rect.2
                    && cursor_pos.y >= rect.1
                    && cursor_pos.y <= rect.3
                {
                    if !o_hovered.is_some() {
                        commands.entity(entity).insert(Hovered);
                        element_found = true;
                    }
                } else {
                    if o_hovered.is_some() {
                        commands.entity(entity).remove::<Hovered>();
                    }
                }
            }
        }
    }
}

fn press_down_system(
    mut commands: Commands,
    query: Query<(Entity, &Clickable), With<Hovered>>,
    mouse_buttons: Res<Input<MouseButton>>,
) {
    for (entity, _) in query.iter() {
        if mouse_buttons.just_pressed(MouseButton::Left) {
            commands.entity(entity).insert(Selected);
        }
    }
}

fn release_system(
    mut commands: Commands,
    query: Query<(Entity, &Clickable, &Selected), With<Hovered>>,
    mouse_buttons: Res<Input<MouseButton>>,
) {
    for (entity, _, _) in query.iter() {
        if mouse_buttons.just_released(MouseButton::Left) {
            commands.entity(entity).insert(Clicked);
        }
    }
}

fn remove_selected_system(
    mut commands: Commands,
    query: Query<(Entity, &Clickable, &Selected)>,
    mouse_buttons: Res<Input<MouseButton>>,
) {
    for (entity, _, _) in query.iter() {
        if mouse_buttons.just_released(MouseButton::Left) {
            commands.entity(entity).remove::<Selected>();
        }
    }
}

//TODO after press down
fn remove_clicked_system(
    mut commands: Commands,
    query: Query<(Entity, &Clickable), With<Clicked>>,
    mouse_buttons: Res<Input<MouseButton>>,
) {
    for (entity, _) in query.iter() {
        if mouse_buttons.just_pressed(MouseButton::Left) {
            commands.entity(entity).remove::<Clicked>();
        }
    }
}
