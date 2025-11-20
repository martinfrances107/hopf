//! A gizmo, where the handles control the generation of a hopf_mesh.
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

use bevy::input::common_conditions::input_pressed;
use bevy::prelude::Cone;
use bevy::{color::palettes::tailwind::*, picking::pointer::PointerInteraction, prelude::*};
use bevy_hopf::HopfPlugin;
use bevy_hopf::hopf::HopfMeshBuilder;
use bevy_mod_mesh_tools::mesh_with_transform;
use bevy_picking::Pickable;

use hopf::sp::SurfacePoint;

fn main() {
    App::new()
        // MeshPickingPlugin is not a default plugin
        .add_plugins((DefaultPlugins, HopfPlugin, MeshPickingPlugin))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (draw_mesh_intersections, rotate, mouse_down_system))
        // .add_systems(Update, draw_cursor)
        .add_systems(
            Update,
            mouse_down_system.run_if(input_pressed(MouseButton::Left)),
        )
        .add_systems(Update, close_on_esc)
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

const SHAPES_X_EXTENT: f32 = 8.0;
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
        lat: 0_f32.to_radians(),
        lon: 0_f32,
    };

    let line_end = SurfacePoint {
        lat: 0_f32.to_radians(),
        lon: 0_f32.to_radians(),
    };

    // The Entity transform will be used SOLEY to set the lat /lon.
    // So here we use mesh_with_transform() to modify the virtex points directly.
    let indicator = mesh_with_transform(
        &Cone::new(indicator_radius, indicator_height).into(),
        // place indicator as it were on an along the z-axes
        // with radius, as if indication were lan 0, long 0
        &Transform::from_xyz(0.0, 0.0, -indicator_ball_radius - indicator_height / 2_f32)
            .with_rotation(Quat::from_euler(
                EulerRot::XYZEx,
                90.0_f32.to_radians(),
                0_f32,
                0_f32,
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

    commands
        .spawn((
            Mesh3d(meshes.add(sphere)),
            MeshMaterial3d(white_matl.clone()),
            Transform::from_translation(origin),
            // .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            // Attach an observer to handle drag events
            Shape,
            IndicatorBall,
        ))
        // Children position determined in relation to the parent transform.
        .with_children(|parent| {
            // Start Indicator ( lat, lon )
            parent.spawn((
                IndicatorHandle,
                Mesh3d(indicator.clone()),
                MeshMaterial3d(indicator_mtl.clone()),
                Transform::default(),
                // Transform::from_rotation(Quat::from_euler(
                //     EulerRot::XYZEx,
                //     0_f32,
                //     line_start.lon,
                //     line_start.lat,
                // )),
            ));

            // End Indicator is ( lat, lon )
            // parent.spawn((
            //     Mesh3d(indicator),
            //     MeshMaterial3d(indicator_mtl),
            //     Transform::from_rotation(Quat::from_euler(
            //         EulerRot::XYZEx,
            //         0_f32,
            //         line_end.lon,
            //         line_end.lat,
            //     )),
            //     IndicatorHandle,
            // ));
        })
        .observe(update_material_on::<Pointer<Over>>(hover_matl.clone()))
        .observe(update_material_on::<Pointer<Out>>(white_matl))
        .observe(update_material_on::<Pointer<Press>>(pressed_matl.clone()))
        .observe(update_material_on::<Pointer<Release>>(hover_matl.clone()))
        .observe(rotate_on_drag);
    // .observe(rotate_handle_on_drag);

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
fn rotate(mut _query: Query<&mut Transform, With<Shape>>, _time: Res<Time>) {
    // for mut transform in &mut query {
    //     transform.rotate_y(time.delta_secs() / 2.);
    // }
}

/// An observer to rotate an entity when it is dragged
fn rotate_on_drag(_drag: On<Pointer<Drag>>, _transforms: Query<&mut Transform>) {
    // let mut transform = transforms.get_mut(drag.entity).unwrap();
    // transform.rotate_y(drag.delta.x * 0.02);
    // transform.rotate_x(drag.delta.y * 0.02);
}

fn mouse_down_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    pointers: Query<&PointerInteraction>,
    indicator_ball: Single<&Children, With<IndicatorBall>>,
    mut handles: Query<(&IndicatorHandle, &mut Transform)>,
) {
    // Is this needed as the system is wrapped in a  runs_if()
    // Could reverse the nested for loops here
    // and search handles first.
    if mouse_button_input.pressed(MouseButton::Left) {
        for normal in pointers
            .iter()
            .filter_map(|interaction| interaction.get_nearest_hit())
            .filter_map(|(_entity, hit)| hit.normal)
        {
            // There is only one hit here.

            // TODO: Select handle based on the proximity to hit point.
            for handle_id in indicator_ball.iter() {
                if let Ok((_indicator_handle, mut transform)) = handles.get_mut(handle_id) {
                    let from = transform.forward().into();
                    println!("surface normal {normal}");
                    println!("forward {}", from);

                    let quat = Quat::from_rotation_arc(from, normal);
                    transform.rotate(quat);
                    println!("transform (after) {}", transform.forward());
                    println!();
                }
            }
        }
    }
}

fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}
