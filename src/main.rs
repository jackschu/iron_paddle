use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use bevy::sprite::MaterialMesh2dBundle;
use bevy_ggrs::{
    AddRollbackCommandExtension, GgrsApp, GgrsPlugin, GgrsSchedule, LocalPlayers, ReadInputs,
    Session,
};
use bevy_matchbox::prelude::*;

use components::*;
use ggrs::{PlayerType, SessionBuilder};
use grid::*;
use input::*;
use physics::*;

mod components;
mod grid;
mod input;
mod physics;
mod util;

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
struct MyWorldCoords(Vec2);

/// true iff player exists at DEPTH
#[derive(Resource)]
pub struct IsDeepPlayer(bool);

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

    app.add_plugins(GgrsPlugin::<GGRSConfig>::default())
        .set_rollback_schedule_fps(FPS)
        .add_systems(ReadInputs, my_cursor_system)
        .rollback_component_with_clone::<Transform>()
        .rollback_component_with_copy::<Ball>()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    filter: "info,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=debug".into(),
                    level: bevy::log::Level::DEBUG,
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true, // behave on wasm
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(ShapePlugin)
        .insert_resource(args)
        .add_state::<AppState>()
        .add_systems(
            OnEnter(AppState::Lobby),
            (lobby_startup, start_matchbox_socket),
        )
        .add_systems(Update, lobby_system.run_if(in_state(AppState::Lobby)))
        .add_systems(OnExit(AppState::Lobby), lobby_cleanup)
        .add_systems(OnEnter(AppState::InGame), setup_scene_system)
        .add_systems(OnEnter(AppState::InGame), setup_grid_system)
        .add_systems(Update, log_ggrs_events.run_if(in_state(AppState::InGame)))
        .add_systems(
            GgrsSchedule,
            (
                paddle_movement,
                ball_movement.after(paddle_movement),
                update_depth_indicator.after(ball_movement),
            ),
        )
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

    let lowest_player = socket.connected_peers().min_by(|l, r| l.0.cmp(&r.0));
    let maybe_own_id = socket.id();

    if maybe_own_id.is_none() {
        return;
    }
    let own_id = maybe_own_id.unwrap();
    commands.insert_resource(IsDeepPlayer(own_id < lowest_player.unwrap()));

    let max_prediction = 12;

    // create a GGRS P2P session
    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(args.players)
        .with_max_prediction_window(max_prediction)
        .expect("REASON")
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
    commands
        .spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(50.).into()).into(),
                material: materials.add(Color::PURPLE.into()),
                transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
                ..default()
            },
            Ball {
                pos: Vec3 {
                    x: 0.,
                    y: 0.,
                    z: 400.,
                },
                speed: Vec3 {
                    x: 150.,
                    y: 200.,
                    z: -400.,
                },
            },
        ))
        .add_rollback();
    for handle in 0..num_players {
        // Rectangle
        commands
            .spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: if handle == 0 {
                            Color::rgba(0.25, 0.25, 0.75, 0.20)
                        } else {
                            Color::rgba(0.25, 0.75, 0.75, 0.20)
                        },
                        custom_size: Some(Vec2::new(100.0, 50.0)),
                        ..default()
                    }, // TODO move transfor
                    transform: Transform::from_translation(Vec3::new(-50., 0., 0.)),
                    ..default()
                },
                Paddle { handle },
            ))
            .add_rollback();
    }
}
