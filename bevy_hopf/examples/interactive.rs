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

use std::f32::consts::PI;

use bevy::{color::palettes::tailwind::*, picking::pointer::PointerInteraction, prelude::*};
use bevy_hopf::HopfPlugin;
use bevy_hopf::hopf::HopfMeshBuilder;
use bevy_mesh::{PrimitiveTopology, VertexAttributeValues};

fn main() {
    App::new()
        // MeshPickingPlugin is not a default plugin
        .add_plugins((DefaultPlugins, HopfPlugin, MeshPickingPlugin))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (draw_mesh_intersections, rotate))
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane.
#[derive(Component)]
struct Shape;

const SHAPES_X_EXTENT: f32 = 10.0;
const Z_EXTENT: f32 = 10.0;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Set up the materials.
    // let white_matl = materials.add(Color::WHITE);
    let white_matl = materials.add(Color::WHITE);
    let ground_matl = materials.add(Color::from(GRAY_300));
    let hover_matl = materials.add(Color::from(CYAN_300));
    let pressed_matl = materials.add(Color::from(YELLOW_300));

    // Initial line/ Initial mesh.
    let line_start = &(10_f64.to_radians(), 0_f64);
    let line_end = &(10_f64.to_radians(), 270_f64.to_radians());

    // Resolution of the mesh is in 2 parameters;
    //
    // Number of loops.
    let n_loops = 27;
    // Number of points per loop.
    let n_points_per_loop = 80;
    let n_tries = 2000;

    let mut hopf_builder = HopfMeshBuilder::new(
        line_start,
        line_end,
        n_points_per_loop,
        n_loops,
        n_tries,
        0.03,
    );

    let hopf_mesh = hopf_builder
        .construct()
        .expect("Failed to construct mesh")
        .build();

    // Create a hair mesh to show the normals.
    let positions = if let Some(VertexAttributeValues::Float32x3(positions)) =
        hopf_mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        positions
    } else {
        panic!("Expected positions to be Float32x3");
    };

    // let normals = hopf_mesh.attribute(Mesh::ATTRIBUTE_NORMAL).as_slice();
    let normals = if let Some(VertexAttributeValues::Float32x3(normals)) =
        hopf_mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
    {
        normals
    } else {
        panic!("Expected normals to be Float32x3");
    };

    // Create a new mesh for the normal lines
    let mut normal_lines = Mesh::new(PrimitiveTopology::LineList, Default::default());

    // For each vertex, create a line from the vertex to vertex + normal
    let mut line_positions = Vec::new();
    for (pos, normal) in positions.iter().zip(normals.iter()) {
        // Scale the normal for better visibility

        // let scaled_normal = *normal * 1.0;
        let scaled_normal = normal;
        // let end_pos = *pos + *scaled_normal;
        let end_pos = [
            pos[0] + 0.1 * scaled_normal[0],
            pos[1] + 0.1 * scaled_normal[1],
            pos[2] + 0.1 * scaled_normal[2],
        ];
        line_positions.push(*pos);
        line_positions.push(end_pos);
    }

    // Insert the line positions as the attribute
    normal_lines.insert_attribute(Mesh::ATTRIBUTE_POSITION, line_positions);

    let sphere = Sphere::default().mesh().ico(5).unwrap();
    let shapes = [meshes.add(sphere), meshes.add(hopf_mesh)];

    let num_shapes = shapes.len();

    // Spawn the shapes. The meshes will be pickable by default.
    for (i, shape) in shapes.into_iter().enumerate() {
        commands
            .spawn((
                Mesh3d(shape),
                MeshMaterial3d(white_matl.clone()),
                Transform::from_xyz(
                    -SHAPES_X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * SHAPES_X_EXTENT,
                    3.0,
                    Z_EXTENT / 2.,
                )
                .with_scale(Vec3::splat(3.0))
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                Shape,
            ))
            .observe(update_material_on::<Pointer<Over>>(hover_matl.clone()))
            .observe(update_material_on::<Pointer<Out>>(white_matl.clone()))
            .observe(update_material_on::<Pointer<Press>>(pressed_matl.clone()))
            .observe(update_material_on::<Pointer<Release>>(hover_matl.clone()))
            .observe(rotate_on_drag);
    }

    let normal_lines_handle = meshes.add(normal_lines);
    commands.spawn((
        Mesh3d(normal_lines_handle),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::from(RED_500),
            unlit: true,
            ..Default::default()
        })),
        Transform::from_xyz(
            -SHAPES_X_EXTENT / 2. + 1 as f32 / (num_shapes - 1) as f32 * SHAPES_X_EXTENT,
            3.0,
            Z_EXTENT / 2.,
        )
        .with_scale(Vec3::splat(3.0))
        .with_rotation(Quat::from_rotation_x(-PI / 4.)),
        Shape,
    ));

    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(ground_matl.clone()),
        Pickable::IGNORE, // Disable picking for the ground plane.
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
