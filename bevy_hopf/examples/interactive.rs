//! A simple 3D scene to demonstrate mesh picking.
//!
//! [`bevy::picking::backend`] provides an API for adding picking hit tests to any entity. To get
//! started with picking 3d meshes, the [`MeshPickingPlugin`] is provided as a simple starting
//! point, especially useful for debugging. For your game, you may want to use a 3d picking backend
//! provided by your physics engine, or a picking shader, depending on your specific use case.
//!
//! [`bevy::picking`] allows you to compose backends together to make any entity on screen pickable
//! with pointers, regardless of how that entity is rendered. For example, `bevy_ui` and
//! `bevy_sprite` provide their own picking backends that can be enabled at the same time as this
//! mesh picking backend. This makes it painless to deal with cases like the UI or sprites blocking
//! meshes underneath them, or vice versa.
//!
//! If you want to build more complex interactions than afforded by the provided pointer events, you
//! may want to use [`MeshRayCast`] or a full physics engine with raycasting capabilities.
//!
//! By default, the mesh picking plugin will raycast against all entities, which is especially
//! useful for debugging. If you want mesh picking to be opt-in, you can set
//! [`MeshPickingSettings::require_markers`] to `true` and add a [`Pickable`] component to the
//! desired camera and target entities.
#![deny(clippy::all)]
#![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::perf)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![allow(clippy::many_single_char_names)]

use core::f32;
use core::f32::consts::PI;

use bevy::prelude::Cone;
use bevy::{color::palettes::tailwind::*, picking::pointer::PointerInteraction, prelude::*};
use bevy_hopf::HopfPlugin;
use bevy_hopf::hopf::HopfMeshBuilder;
use bevy_mesh::{ConeAnchor, ConeMeshBuilder};
use bevy_mod_mesh_tools::mesh_with_transform;
use bevy_picking::Pickable;

use hopf::sp::SurfacePoint;

#[derive(Component)]
struct IndicatorState {
    start: SurfacePoint,
    end: SurfacePoint,
    origin: Vec3,
    radius: f32,
    active_handle: Option<Entity>,
}

impl IndicatorState {
    fn new(start: SurfacePoint, end: SurfacePoint, origin: Vec3, radius: f32) -> Self {
        Self {
            start,
            end,
            origin,
            radius,
            active_handle: None,
        }
    }
}

fn main() {
    App::new()
        // MeshPickingPlugin is not a default plugin
        .add_plugins((DefaultPlugins, HopfPlugin, MeshPickingPlugin))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (draw_mesh_intersections, rotate))
        .add_systems(Update, draw_cursor)
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane.
#[derive(Component)]
struct Shape;

#[derive(Component)]
struct IndicatorHandle;

#[derive(Component)]
struct IndicatorBall;

#[derive(Component)]
struct Ground;

/// A marker component, allow stylization of the output mesh
#[derive(Component)]
struct Hopf;

const SHAPES_X_EXTENT: f32 = 10.0;
const Z_EXTENT: f32 = 10.0;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Materials.
    let white_matl = materials.add(Color::WHITE);
    let ground_matl = materials.add(Color::from(GRAY_300));
    let hover_matl = materials.add(Color::from(CYAN_300));
    let pressed_matl = materials.add(Color::from(YELLOW_300));
    let indicator_mtl = materials.add(Color::from(CYAN_300));
    let mut hopf_white_matl: StandardMaterial = Color::WHITE.into();
    // Double sided materials.
    hopf_white_matl.cull_mode = None;
    hopf_white_matl.double_sided = true;
    let hopf_white_matl = materials.add(hopf_white_matl);
    let mut hopf_hover_matl: StandardMaterial = Color::from(CYAN_300).into();
    hopf_hover_matl.cull_mode = None;
    hopf_hover_matl.double_sided = true;
    let hopf_hover_matl = materials.add(hopf_hover_matl);
    let mut hopf_pressed_matl: StandardMaterial = Color::from(YELLOW_300).into();
    hopf_pressed_matl.cull_mode = None;
    hopf_pressed_matl.double_sided = true;
    let hopf_pressed_matl = materials.add(hopf_pressed_matl);

    // Gizmo
    let indicator_ball_radius = 2.0;
    let indicator_height = 0.5;
    let indicator_radius = 0.25; // as aspect ration of 0.5;

    // North pole
    let line_start = SurfacePoint {
        lat: 45_f32.to_radians(),
        lon: 0_f32,
    };

    let line_end = SurfacePoint {
        lat: 0_f32.to_radians(),
        lon: 0_f32.to_radians(),
    };

    let indicator = mesh_with_transform(
        &Cone::new(indicator_radius, indicator_height).into(),
        // place indicator as it were on an along the x axes
        // with radius, as if indication were lan 0, long 0
        &Transform::from_xyz(indicator_ball_radius + indicator_height / 2_f32, 0.0, 0.0)
            .with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                0_f32,
                0_f32,
                90.0_f32.to_radians(),
            )),
    )
    .unwrap();
    let indicator = meshes.add(indicator);

    let sphere = Sphere::new(indicator_ball_radius).mesh().ico(5).unwrap();

    // Spherical selector
    let i = 0;
    // Origin of the gizmo
    let origin = Vec3::new(
        -SHAPES_X_EXTENT / 2. + i as f32 / (2 - 1) as f32 * SHAPES_X_EXTENT,
        3.0,
        Z_EXTENT / 2.,
    );

    let indicator_ball = commands
        .spawn((
            Mesh3d(meshes.add(sphere)),
            MeshMaterial3d(white_matl.clone()),
            Transform::from_translation(origin),
            // .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            // Attach an observer to handle drag events
            Shape,
            IndicatorBall,
            IndicatorState::new(
                line_start.clone(),
                line_end.clone(),
                origin,
                indicator_ball_radius,
            ),
        ))
        // Children be offset bt the parent transform
        .with_children(|parent| {
            // Start Indicator ( lat, lon )
            parent.spawn((
                Mesh3d(indicator.clone()),
                MeshMaterial3d(indicator_mtl.clone()),
                Transform::from_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    0_f32,
                    line_start.lon,
                    line_start.lat,
                )),
                IndicatorHandle,
            ));

            // End Indicator is ( lat, lon )
            parent.spawn((
                Mesh3d(indicator),
                MeshMaterial3d(indicator_mtl),
                Transform::from_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    0_f32,
                    line_end.lon,
                    line_end.lat,
                )),
                IndicatorHandle,
            ));
        })
        .observe(update_material_on::<Pointer<Over>>(hover_matl.clone()))
        .observe(update_material_on::<Pointer<Out>>(white_matl))
        .observe(update_material_on::<Pointer<Press>>(pressed_matl.clone()))
        .observe(update_material_on::<Pointer<Release>>(hover_matl.clone()))
        .observe(rotate_on_drag)
        .observe(rotate_handle_on_drag);

    let line_start = SurfacePoint {
        lat: 10_f32.to_radians(),
        lon: 0_f32,
    };

    let line_end = SurfacePoint {
        lat: 10_f32.to_radians(),
        lon: 270_f32.to_radians(),
    };

    // Hopf Object
    let hopf_builder = HopfMeshBuilder::new(&line_start, &line_end, 27, 2000);

    let hopf_mesh = hopf_builder
        .construct::<40>()
        .expect("Failed to construct mesh")
        .build();

    // Hopf mesh
    let i = 1;
    commands
        .spawn((
            Mesh3d(meshes.add(hopf_mesh)),
            MeshMaterial3d(hopf_white_matl.clone()),
            Transform::from_xyz(
                -SHAPES_X_EXTENT / 2. + i as f32 / (2 - 1) as f32 * SHAPES_X_EXTENT,
                3.0,
                Z_EXTENT / 2.,
            )
            .with_scale(Vec3::splat(0.8))
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
            Hopf,
        ))
        .observe(update_material_on::<Pointer<Over>>(hopf_hover_matl.clone()))
        .observe(update_material_on::<Pointer<Out>>(hopf_white_matl.clone()))
        .observe(update_material_on::<Pointer<Press>>(
            hopf_pressed_matl.clone(),
        ))
        .observe(update_material_on::<Pointer<Release>>(
            hopf_hover_matl.clone(),
        ))
        .observe(rotate_on_drag);

    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(ground_matl.clone()),
        Pickable::IGNORE, // Disable picking for the ground plane.
        Ground,
    ));

    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));

    // Instructions
    commands.spawn((
        Text::new("Hover over the shapes to pick them\nDrag to rotate"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

/// Returns an observer that updates the entity's material to the one specified.
fn update_material_on<E: EntityEvent>(
    new_material: Handle<StandardMaterial>,
) -> impl Fn(On<E>, Query<&mut MeshMaterial3d<StandardMaterial>>) {
    // An observer closure that captures `new_material`. We do this to avoid needing to write four
    // versions of this observer, each triggered by a different event and with a different hardcoded
    // material. Instead, the event type is a generic, and the material is passed in.
    move |event, mut query| {
        if let Ok(mut material) = query.get_mut(event.event_target()) {
            material.0 = new_material.clone();
        }
    }
}

/// A system that draws hit indicators for every pointer.
fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 0.05, RED_500);
        gizmos.arrow(point, point + normal.normalize() * 0.5, PINK_100);
    }
}

/// A system that rotates all shapes.
fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() / 2.);
    }
}

/// An observer to rotate an entity when it is dragged
fn rotate_on_drag(drag: On<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
    let mut transform = transforms.get_mut(drag.entity).unwrap();
    transform.rotate_y(drag.delta.x * 0.02);
    transform.rotate_x(drag.delta.y * 0.02);
}

fn rotate_handle_on_drag(
    drag: On<Pointer<Drag>>,
    ib_query: Query<(&IndicatorBall, &Children)>,
    mut child_query: Query<(&IndicatorHandle, &mut Transform)>,
) {
    // use scaled delta values to create rotation updates from start handle
    let lat_change = drag.delta.x * 0.02;
    let lon_change = drag.delta.y * 0.02;

    // FIX currently update both children must seleect base on proximity and keypress.
    for (_, children) in ib_query {
        for child in children {
            if let Ok((_, mut handle_transform)) = child_query.get_mut(*child) {
                // TODO two ratations lead to two separte Quat
                // make this one once stable
                handle_transform.rotate_z(lat_change);
                handle_transform.rotate_y(lon_change);
            }
        }
    }
}

/// DONT'T COPY: This is really laggy ( it was taken from the examples!)
fn draw_cursor(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    ground: Single<&GlobalTransform, With<Ground>>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position()
        // Calculate a ray pointing from the camera into the world based on the cursor's position.
        && let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position)
        // Calculate if and at what distance the ray is hitting the ground plane.
        && let Some(distance) =
            ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
    {
        let point = ray.get_point(distance);

        // Draw a circle just above the ground plane at that position.
        gizmos.circle(
            Isometry3d::new(
                point + ground.up() * 0.01,
                Quat::from_rotation_arc(Vec3::Z, ground.up().as_vec3()),
            ),
            0.2,
            Color::WHITE,
        );
    }
}
