//! Basic example rendering an amoeba.
//! `cargo run --example interactive`
#![recursion_limit = "256"]

use bevy::{
    app::ScheduleRunnerPlugin,
    color::palettes::css::GRAY,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    picking::{backend::HitData, pointer::PointerInteraction},
    prelude::*,
};

use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_softbody2d::{
    SoftBody2dMaterial, SoftBody2dNode, SoftBodyMaterial2dPlugin, SoftBodyMaterialUniform,
    SoftBodyNodes, SoftBodyPlugin,
};

/// Number of vertices for the softbody rendering.
const N1: usize = 128;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 10.0)),
        DefaultPlugins,
        WireframePlugin::default(),
        EguiPlugin::default(),
        WorldInspectorPlugin::new(),
        FpsOverlayPlugin {
            config: FpsOverlayConfig::default(),
        },
        SoftBodyPlugin,
        SoftBodyMaterial2dPlugin::<N1>::default(),
        MeshPickingPlugin,
    ))
    .insert_resource(WireframeConfig {
        default_color: GRAY.into(),
        ..default()
    })
    .init_resource::<CustomSoftBodyAssets<N1>>()
    .init_resource::<CustomSoftBodyNodeAssets>()
    .init_resource::<CommonAssets>()
    .insert_resource(ClearColor(Color::WHITE))
    .add_systems(Startup, setup)
    // .add_systems(FixedUpdate, CustomSoftBodyNode::fixed_update)
    .run();
}

/// Spawn in many soft bodies.
fn setup(mut commands: Commands) {
    commands.spawn(MainCamera);
    commands.spawn(DirectionalLight::default());
    commands.spawn(InteractionCanvas);

    let z = -0.05;
    commands.spawn((
        CustomSoftBody::<N1>,
        Transform {
            translation: Vec3::new(0.0, 0.0, z),
            scale: Vec3::splat(1.0),
            ..default()
        },
    ));
}

/// Setup main camera.
#[derive(Component, Clone, Debug, Reflect)]
#[require(
    Camera3d::default(),
    Projection::Perspective(PerspectiveProjection {
        ..default()
    }),
    Transform {
        translation: Vec3::new(0.0, 0.0, 6.0),
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
            mesh: world.add_asset(Circle { radius: 0.2 }),
            material: world.add_asset(StandardMaterial {
                base_color: Color::BLACK.into(),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        }
    }
}

/// Drag offset in screen space.
#[derive(Component, Clone, Debug, Reflect)]
struct DragOffset(pub Vec3);

/// Custom soft body node.
#[derive(Component, Reflect)]
#[require(Name::new("SoftBodyNode"), Pickable)]
#[component(on_add = Self::on_add)]
struct CustomSoftBodyNode {
    radius: f32,
}
impl CustomSoftBodyNode {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let CustomSoftBodyNodeAssets { mesh, material } =
            world.resource::<CustomSoftBodyNodeAssets>().clone();
        let radius = world.entity(context.entity).components::<&Self>().radius;
        world
            .commands()
            .entity(context.entity)
            .insert((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                SoftBody2dNode { radius },
            ))
            .observe(Self::on_drag_start)
            .observe(Self::on_drag_end)
            .observe(Self::on_drag);
    }
    fn on_drag_start(drag: On<Pointer<DragStart>>, mut commands: Commands) {
        info!("on_drag_start");
        let Some(world_position) = drag.hit.position else {
            return;
        };
        commands
            .entity(drag.entity)
            .insert(DragOffset(world_position));
    }
    fn get_matching_hit(entity: Entity, interactions: &PointerInteraction) -> Option<HitData> {
        for (hit_entity, hit) in interactions.iter() {
            if *hit_entity == entity {
                return Some(hit.clone());
            }
        }
        None
    }
    fn on_drag(
        drag: On<Pointer<Drag>>,
        mut nodes: Query<&mut Transform, With<Self>>,
        interactions: Single<&PointerInteraction>,
    ) {
        let Some(hit) = Self::get_matching_hit(drag.entity, &interactions) else {
            return;
        };
        let Some(hit_position) = hit.position else {
            return;
        };
        let Ok(mut transform) = nodes.get_mut(drag.entity) else {
            return;
        };
        transform.translation.x = hit_position.x;
        transform.translation.y = hit_position.y;
    }
    fn on_drag_end(click: On<Pointer<DragEnd>>, mut commands: Commands) {
        info!("on_drag_end");
        commands.entity(click.entity).remove::<DragOffset>();
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[require(
    Name::new("InteractionCanvas"),
    Pickable,
    Transform {
        translation: Vec3::new(0.0, 0.0, -1.0),
        scale: Vec3::splat(20.0),
        ..default()
    }
)]
#[component(on_add = Self::on_add)]
struct InteractionCanvas;
impl InteractionCanvas {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let CommonAssets {
            quad,
            background_material,
        } = world.resource::<CommonAssets>().clone();
        world
            .commands()
            .entity(context.entity)
            .insert((Mesh3d(quad), MeshMaterial3d(background_material)))
            .observe(Self::on_click)
            // .observe(Self::on_drag_over)
            ;
    }
    fn on_click(
        click: On<Pointer<Click>>,
        mut softbody: Single<&mut SoftBodyNodes<N1>, With<CustomSoftBody<N1>>>,
        mut commands: Commands,
    ) {
        if click.button != PointerButton::Primary {
            return;
        }
        let Some(hit_position) = click.hit.position else {
            return;
        };
        let entity = commands
            .spawn((
                CustomSoftBodyNode { radius: 1.0 },
                Transform::from_translation(hit_position + Vec3::Z * 0.1),
                Visibility::default(),
            ))
            .id();
        softbody.0.push(entity);
    }
    // fn on_drag_over(
    //     drag: On<Pointer<DragOver>>,
    //     mut transforms: Query<&mut Transform, With<DragOffset>>,
    // ) {
    //     println!("on_drag_over");
    //     // dbg!(drag.entity);
    //     if let Ok(mut transform) = transforms.get_mut(drag.entity) {
    //         let Some(position) = drag.hit.position else {
    //             return;
    //         };
    //         println!("hit position");
    //         transform.translation = position;
    //     }
    // }
}

#[derive(Component, Reflect, Copy, Clone)]
#[component(on_add = Self::on_add)]
#[require(Name::new("CustomSoftBody"), Pickable::IGNORE)]
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
        let entities = [(
            CustomSoftBodyNode { radius: 1.0 },
            Transform {
                translation: Vec3::ZERO * scale + translation,
                ..default()
            },
        )]
        .into_iter()
        .map(|bundle| world.commands().spawn(bundle).id())
        .collect();

        let CustomSoftBodyAssets { material, .. } =
            world.resource::<CustomSoftBodyAssets<N>>().clone();
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

#[derive(Resource, Reflect, Clone)]
struct CommonAssets {
    quad: Handle<Mesh>,
    background_material: Handle<StandardMaterial>,
}
impl FromWorld for CommonAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            quad: world.add_asset(Rectangle {
                half_size: Vec2::new(0.5, 0.5),
            }),
            background_material: world.add_asset(StandardMaterial::from_color(Color::WHITE)),
        }
    }
}
