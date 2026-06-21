//! Basic example rendering an amoeba.
//! `cargo run --example basic`
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
const N1: usize = 32;
const N2: usize = 64;

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
        SoftBodyMaterial2dPlugin::<N2>::default(),
    ))
    .insert_resource(WireframeConfig {
        default_color: GRAY.into(),
        ..default()
    })
    .init_resource::<CustomSoftBodyAssets<N1>>()
    .init_resource::<CustomSoftBodyAssets<N2>>()
    .init_resource::<CustomSoftBodyNodeAssets>()
    .insert_resource(ClearColor(Color::WHITE))
    .add_systems(Startup, setup)
    .add_systems(FixedUpdate, CustomSoftBodyNode::fixed_update)
    .add_systems(
        Update,
        (
            toggle_wireframe.run_if(input_just_pressed(KeyCode::Space)),
            rotate.run_if(input_toggle_active(false, KeyCode::KeyR)),
        ),
    )
    .run();
}

/// Spawn in many soft bodies.
fn setup(mut commands: Commands) {
    commands.spawn(MainCamera);
    commands.spawn(DirectionalLight::default());

    let z = -0.1;
    let x_step = 1.6;
    let y_step = 1.6;
    let x_total = 8;
    let y_total = 8;

    for y in 0..y_total {
        for x in 0..x_total / 2 {
            commands.spawn((
                CustomSoftBody::<N1>,
                Transform {
                    translation: Vec3::new(
                        (x - x_total / 2) as f32 * x_step,
                        (y - y_total / 2) as f32 * y_step,
                        z + ((x + y * x_total) as f32 * 0.0001),
                    ),
                    scale: Vec3::splat(1.0),
                    ..default()
                },
            ));
        }
    }

    for y in 0..y_total {
        for x in x_total / 2..x_total {
            commands.spawn((
                CustomSoftBody::<N2>,
                Transform {
                    translation: Vec3::new(
                        (x - x_total / 2) as f32 * x_step,
                        (y - y_total / 2) as f32 * y_step,
                        z + ((x + y * x_total) as f32 * 0.0001),
                    ),
                    scale: Vec3::splat(1.25),
                    ..default()
                },
            ));
        }
    }
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
struct CustomSoftBodyNodeAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}
impl FromWorld for CustomSoftBodyNodeAssets {
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
#[require(Name::new("SoftBodyNode"))]
#[component(on_add = CustomSoftBodyNode::on_add)]
struct CustomSoftBodyNode {
    radius: f32,
}
impl CustomSoftBodyNode {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let CustomSoftBodyNodeAssets { mesh, material } =
            world.resource::<CustomSoftBodyNodeAssets>().clone();
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
            transform.translation.x += alpha * (time.elapsed_secs() * omega + phi).cos();
            transform.translation.y += alpha * (time.elapsed_secs() * omega + phi).sin();
        }
    }
}

#[derive(Component, Reflect, Copy, Clone)]
#[component(on_add = CustomSoftBody::<N>::on_add)]
#[require(Name::new("SoftBody"))]
struct CustomSoftBody<const N: usize>;
impl<const N: usize> CustomSoftBody<N> {
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
                CustomSoftBodyNode { radius: 0.6 },
                Transform {
                    translation: Vec3::new(0.1, -0.1, 0.0) * scale + translation,
                    ..default()
                },
            ),
            (
                CustomSoftBodyNode { radius: 0.5 },
                Transform {
                    translation: Vec3::new(0.3, 0.3, 0.0) * scale + translation,
                    ..default()
                },
            ),
            (
                CustomSoftBodyNode { radius: 0.4 },
                Transform {
                    translation: Vec3::new(-0.2, -0.2, 0.0) * scale + translation,
                    ..default()
                },
            ),
        ]
        .into_iter()
        .map(|bundle| world.commands().spawn(bundle).id())
        .collect();

        let CustomSoftBodyAssets { material } = world.resource::<CustomSoftBodyAssets<N>>().clone();
        world
            .commands()
            .entity(context.entity)
            .insert((SoftBodyNodes::<N>(entities), MeshMaterial3d(material)));
    }
}

/// Global handles to soft body assets to enable GPU instancing.
#[derive(Resource, Reflect, Clone)]
struct CustomSoftBodyAssets<const N: usize> {
    material: Handle<SoftBody2dMaterial<N>>,
}
impl<const N: usize> CustomSoftBodyAssets<N> {
    fn get_color_texture(world: &World) -> Handle<Image> {
        match N {
            N1 => world.load_asset("textures/bubble_bl.png"),
            N2 => world.load_asset("textures/bubble_or.png"),
            _ => unreachable!(),
        }
    }
}
impl<const N: usize> FromWorld for CustomSoftBodyAssets<N> {
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
