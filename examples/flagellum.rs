//! Basic example rendering an amoeba.
//! `cargo run --example flagellum`
#![recursion_limit = "256"]
use std::f32::consts::PI;

use bevy::{
    color::palettes::css::GRAY,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    input::common_conditions::{input_just_pressed, input_toggle_active},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};

use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_softbody2d::{
    SoftBody2dMaterial, SoftBody2dNode, SoftBodyMaterial2dPlugin, SoftBodyMaterialUniform,
    SoftBodyNodes, SoftBodyPlugin,
};

/// Number of vertices for the softbody rendering.
const N1: usize = 64;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        WireframePlugin::default(),
        EguiPlugin::default(),
        WorldInspectorPlugin::new(),
        FpsOverlayPlugin {
            config: FpsOverlayConfig::default(),
        },
        SoftBodyPlugin,
        SoftBodyMaterial2dPlugin::<N1>::default(),
    ))
    .insert_resource(WireframeConfig {
        default_color: GRAY.into(),
        ..default()
    })
    .init_resource::<FlagellumAssets<N1>>()
    .init_resource::<FlagellumNodeAssets>()
    .insert_resource(ClearColor(Color::WHITE))
    .add_systems(Startup, setup)
    .add_systems(FixedUpdate, FlagellumNode::fixed_update)
    .add_systems(
        Update,
        (
            toggle_wireframe.run_if(input_just_pressed(KeyCode::Space)),
            rotate.run_if(input_toggle_active(false, KeyCode::KeyR)),
        ),
    )
    .run();
}

/// Spawn in a flagellum
fn setup(mut commands: Commands) {
    commands.spawn(MainCamera);
    commands.spawn(DirectionalLight::default());

    commands.spawn((
        Flagellum::<N1>,
        Transform {
            scale: Vec3::splat(2.0),
            ..default()
        },
    ));
}

fn toggle_wireframe(mut wireframe_config: ResMut<WireframeConfig>) {
    wireframe_config.global = !wireframe_config.global;
}

fn rotate(
    mut query: Query<&mut Transform, With<MeshMaterial3d<SoftBody2dMaterial<N1>>>>,
    time: Res<Time>,
) {
    for mut transform in &mut query {
        transform.rotate_z(time.delta_secs() / 2.0);
    }
}

/// Setup main camera.
#[derive(Component, Reflect)]
#[require(
    Camera3d::default(),
    Projection::Perspective(PerspectiveProjection {
        fov: PI / 2.0,
        near: 0.1,
        far: 2000.,
        ..default()
    }),
    Transform {
        translation: Vec3::new(0.0, 0.0, 8.0),
        ..default()
    })]
struct MainCamera;

/// Assets for spawning soft body nodes.
#[derive(Resource, Reflect, Clone)]
struct FlagellumNodeAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}
impl FromWorld for FlagellumNodeAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh: world.add_asset(Circle { radius: 0.1 }),
            material: world.add_asset(StandardMaterial {
                base_color: Color::BLACK.into(),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        }
    }
}

/// Custom soft body node.
#[derive(Component, Reflect)]
#[require(Name::new("FlagellumNode"))]
#[component(on_add = FlagellumNode::on_add)]
struct FlagellumNode {
    radius: f32,
}
impl FlagellumNode {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let FlagellumNodeAssets { mesh, material } =
            world.resource::<FlagellumNodeAssets>().clone();
        let radius = world.entity(context.entity).get::<Self>().unwrap().radius;
        world.commands().entity(context.entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            SoftBody2dNode { radius },
        ));
    }
    /// Make the nodes move around.
    pub fn fixed_update(mut query: Query<&mut Transform, With<Self>>, time: Res<Time>) {
        let alpha = 0.0015;
        let omega = 2.0;
        for (i, mut transform) in query.iter_mut().enumerate() {
            let phi = i as f32;
            transform.translation.y += alpha * (time.elapsed_secs() * omega + phi).sin();
        }
    }
}

#[derive(Component, Reflect, Copy, Clone)]
#[component(on_add = Flagellum::<N>::on_add)]
#[require(Name::new("Flagellum"))]
struct Flagellum<const N: usize>;
impl<const N: usize> Flagellum<N> {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let Transform {
            translation, scale, ..
        } = world
            .entity(context.entity)
            .get::<Transform>()
            .unwrap()
            .clone();
        let entities = [
            (
                FlagellumNode { radius: 0.2 },
                Transform {
                    translation: Vec3::new(-0.6, 0.0, 0.0) * scale + translation,
                    ..default()
                },
            ),
            (
                FlagellumNode { radius: 0.2 },
                Transform {
                    translation: Vec3::new(-0.2, 0.0, 0.0) * scale + translation,
                    ..default()
                },
            ),
            (
                FlagellumNode { radius: 0.2 },
                Transform {
                    translation: Vec3::new(0.0, 0.0, 0.0) * scale + translation,
                    ..default()
                },
            ),
            (
                FlagellumNode { radius: 0.2 },
                Transform {
                    translation: Vec3::new(0.2, 0.0, 0.0) * scale + translation,
                    ..default()
                },
            ),
            (
                FlagellumNode { radius: 0.2 },
                Transform {
                    translation: Vec3::new(0.6, 0.0, 0.0) * scale + translation,
                    ..default()
                },
            ),
        ]
        .into_iter()
        .map(|bundle| world.commands().spawn(bundle).id())
        .collect();

        let FlagellumAssets { material } = world.resource::<FlagellumAssets<N>>().clone();
        world
            .commands()
            .entity(context.entity)
            .insert((SoftBodyNodes::<N>(entities), MeshMaterial3d(material)));
    }
}

/// Global handles to soft body assets to enable GPU instancing.
#[derive(Resource, Reflect, Clone)]
struct FlagellumAssets<const N: usize> {
    material: Handle<SoftBody2dMaterial<N>>,
}
impl<const N: usize> FlagellumAssets<N> {
    fn get_color_texture(world: &World) -> Handle<Image> {
        match N {
            N1 => world.load_asset("textures/bubble_bw.png"),
            _ => unreachable!(),
        }
    }
}
impl<const N: usize> FromWorld for FlagellumAssets<N> {
    fn from_world(world: &mut World) -> Self {
        let material = SoftBody2dMaterial::<N> {
            uniforms: SoftBodyMaterialUniform {
                color: Color::WHITE.into(),
            },
            color_texture: Some(Self::get_color_texture(world)),
            alpha_mode: AlphaMode::Blend,
            ..FromWorld::from_world(world)
        };
        Self {
            material: world.add_asset(material),
        }
    }
}
