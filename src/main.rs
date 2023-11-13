use bevy::core::Zeroable;
use bevy::log::LogPlugin;
use bevy::window::PrimaryWindow;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_ggrs::{
    AddRollbackCommandExtension, GgrsPlugin, GgrsSchedule, PlayerInputs, Rollback, Session,
};
use bevy_matchbox::prelude::*;
use bytemuck::Pod;
use ggrs::{Config, PlayerHandle, SessionBuilder};

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
struct MyWorldCoords(Vec2);

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

const FPS: usize = 60;

#[derive(Debug, Clone, Resource)]
pub struct Args {
    pub matchbox: String,
    pub room: Option<String>,
    pub players: usize,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            //matchbox: "ws://127.0.0.1:3536".to_owned(),
            //            matchbox: "wss://xcfj9nmjx9.execute-api.us-east-1.amazonaws.com/devbox".to_owned(),
            matchbox: "wss://nlezrk177f.execute-api.us-east-1.amazonaws.com/prod".to_owned(),
            room: None,
            players: 2,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Pod, Zeroable, Debug)]
pub struct BoxInput {
    pub inp: i64,
    pub inp2: i64,
}

/// You need to define a config struct to bundle all the generics of GGRS. You can safely ignore
/// `State` and leave it as u8 for all GGRS functionality.
/// TODO: Find a way to hide the state type.
#[derive(Debug)]
pub struct GGRSConfig;
impl Config for GGRSConfig {
    type Input = BoxInput;
    type State = u8;
    type Address = PeerId;
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Lobby,
    InGame,
}
// Marker components for UI
#[derive(Component)]
struct LobbyText;
#[derive(Component)]
struct LobbyUI;

fn main() {
    // todo can get from query string?
    let args = Args::default();
    let mut app = App::new();

    GgrsPlugin::<GGRSConfig>::new()
        // define frequency of rollback game logic update
        .with_update_frequency(FPS)
        // define system that returns inputs given a player handle, so GGRS can send the inputs
        // around
        .with_input_system(my_cursor_system)
        // register types of components AND resources you want to be rolled back
        .register_rollback_component::<Transform>()
        //        .register_rollback_component::<Velocity>()
        //        .register_rollback_resource::<FrameCount>()
        // make it happen in the bevy app
        .build(&mut app);

    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                filter: "info,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=debug".into(),
                level: bevy::log::Level::DEBUG,
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true, // behave on wasm
                    ..default()
                }),
                ..default()
            }),
    )
    .insert_resource(args)
    .add_state::<AppState>()
    .add_systems(
        OnEnter(AppState::Lobby),
        (lobby_startup, start_matchbox_socket),
    )
    .add_systems(Update, lobby_system.run_if(in_state(AppState::Lobby)))
    .add_systems(OnExit(AppState::Lobby), lobby_cleanup)
    .add_systems(OnEnter(AppState::InGame), setup_scene_system)
    .add_systems(Update, log_ggrs_events.run_if(in_state(AppState::InGame)))
    .add_systems(GgrsSchedule, paddle_movement)
    .run();
}

fn lobby_startup(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            background_color: Color::rgb(0.43, 0.41, 0.38).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    text: Text::from_section(
                        "Entering lobby...",
                        TextStyle {
                            font_size: 96.,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    ),
                    ..default()
                })
                .insert(LobbyText);
        })
        .insert(LobbyUI);
}
fn lobby_cleanup(query: Query<Entity, With<LobbyUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn lobby_system(
    mut app_state: ResMut<NextState<AppState>>,
    args: Res<Args>,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut commands: Commands,
    mut query: Query<&mut Text, With<LobbyText>>,
) {
    // regularly call update_peers to update the list of connected peers
    for (peer, new_state) in socket.update_peers() {
        // you can also handle the specific dis(connections) as they occur:
        match new_state {
            PeerState::Connected => info!("peer {peer} connected"),
            PeerState::Disconnected => info!("peer {peer} disconnected"),
        }
    }

    let connected_peers = socket.connected_peers().count();
    let remaining = args.players - (connected_peers + 1);
    query.single_mut().sections[0].value = format!("Waiting for {remaining} more player(s)",);
    if remaining > 0 {
        return;
    }

    info!("All peers have joined, going in-game");

    // extract final player list
    let players = socket.players();

    let max_prediction = 12;

    // create a GGRS P2P session
    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(args.players)
        .with_max_prediction_window(max_prediction)
        //        .expect("REASON")
        .with_input_delay(2)
        .with_fps(FPS)
        .expect("invalid fps");

    for (i, player) in players.into_iter().enumerate() {
        sess_build = sess_build
            .add_player(player, i)
            .expect("failed to add player");
    }

    let channel = socket.take_channel(0).unwrap();

    // start the GGRS session
    let sess = sess_build
        .start_p2p_session(channel)
        .expect("failed to start session");

    commands.insert_resource(Session::P2P(sess));

    // transition to in-game state
    app_state.set(AppState::InGame);
}

fn log_ggrs_events(mut session: ResMut<Session<GGRSConfig>>) {
    match session.as_mut() {
        Session::P2P(s) => {
            for event in s.events() {
                info!("GGRS Event: {event:?}");
            }
        }
        _ => panic!("This example focuses on p2p."),
    }
}

fn start_matchbox_socket(mut commands: Commands, args: Res<Args>) {
    let room_id = match &args.room {
        Some(id) => id.clone(),
        None => format!("?next={}", &args.players),
    };

    let room_url = format!("{}/{}", &args.matchbox, room_id);
    info!("connecting to matchbox server: {room_url:?}");

    commands.insert_resource(MatchboxSocket::new_ggrs(room_url));
}

#[derive(Default, Component)]
struct Paddle {
    pub handle: usize,
}

#[derive(Component)]
struct Ball;

fn setup_scene_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    session: Res<Session<GGRSConfig>>,
) {
    let num_players = match &*session {
        Session::SyncTest(s) => s.num_players(),
        Session::P2P(s) => s.num_players(),
        Session::Spectator(s) => s.num_players(),
    };
    commands.init_resource::<MyWorldCoords>();
    commands.spawn((Camera2dBundle::default(), MainCamera));

    // Circle
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(50.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
            ..default()
        },
        Ball,
    ));

    for handle in 0..num_players {
        // Rectangle
        commands
            .spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: if handle == 0 {
                            Color::rgba(0.25, 0.25, 0.75, 0.50)
                        } else {
                            Color::MAROON
                        },
                        custom_size: Some(Vec2::new(50.0, 100.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(-50., 0., 0.)),
                    ..default()
                },
                Paddle { handle },
            ))
            .add_rollback();
    }
}

fn my_cursor_system(
    _handle: In<PlayerHandle>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> BoxInput {
    let default = BoxInput { inp: 0, inp2: 0 };
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    if let Ok((camera, camera_transform)) = q_camera.get_single() {
        // There is only one primary window, so we can similarly get it from the query:
        let window = q_window.single();

        // check if the cursor is inside the window and get its position
        // then, ask bevy to convert into world coordinates, and truncate to discard Z
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            BoxInput {
                inp: (world_position.x * 1000.0).floor() as i64,
                inp2: (world_position.y * 1000.0).floor() as i64,
            }
        } else {
            return default;
        }
    } else {
        return default;
    }
}
/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn paddle_movement(
    mut query: Query<(&mut Transform, &mut Paddle), With<Rollback>>,
    inputs: Res<PlayerInputs<GGRSConfig>>,
) {
    for (mut transform, p) in query.iter_mut() {
        let input = inputs[p.handle].0;
        transform.translation.x = (input.inp as f32) / 1000.0;
        transform.translation.y = (input.inp2 as f32) / 1000.0;
    }
}
