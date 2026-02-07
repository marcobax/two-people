//! TWO PEOPLE - Fast-paced "Which type are you?" game
//! CHOOSE FAST!

use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
    window::PrimaryWindow,
};
use rand::Rng;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool, Row};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

// SETTINGS
const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;
const QUESTION_TIME: f32 = 5.0;
const HURRY_TIME: f32 = 2.0;

// COLORS - Vibrant!
const BG_COLOR: Color = Color::srgb(0.06, 0.06, 0.10);
const CARD_LEFT: Color = Color::srgb(1.0, 0.3, 0.4);
const CARD_RIGHT: Color = Color::srgb(0.25, 0.6, 1.0);
const TIMER_NORMAL: Color = Color::WHITE;
const TIMER_HURRY: Color = Color::srgb(1.0, 0.2, 0.2);
const TEXT_YELLOW: Color = Color::srgb(1.0, 0.95, 0.0);
const RESULT_GREEN: Color = Color::srgb(0.2, 1.0, 0.5);

// Sizes
const CARD_W: f32 = 260.0;
const CARD_H: f32 = 360.0;
const CARD_GAP: f32 = 320.0;
const HOVER_SCALE: f32 = 1.1;

// Components
#[derive(Component)]
struct Card {
    choice: Choice,
    base_y: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Choice {
    Left,
    Right,
}

#[derive(Component)]
struct TitleText;
#[derive(Component)]
struct TimerDisplay;
#[derive(Component)]
struct HurryText;
#[derive(Component)]
struct ResultDisplay;
#[derive(Component)]
struct CardLabel {
    choice: Choice,
}
#[derive(Component)]
struct Particle {
    vel: Vec2,
    phase: f32,
    spin: f32,
}

#[derive(Component)]
struct BgShape {
    spin_speed: f32,
    pulse_speed: f32,
    phase: f32,
}
#[derive(Component)]
struct Pulse {
    speed: f32,
}

#[derive(Component)]
struct Firework {
    vel: Vec2,
    life: f32,
    max_life: f32,
    color_hue: f32,
}

#[derive(Component)]
struct ReplayInstruction;

#[derive(Component)]
struct StatsDisplay;

#[derive(Component)]
struct UhOhText;

#[derive(Resource, Default)]
struct DbStats {
    loaded: bool,
    total_players: i64,
    avg_left_pct: f64,
}

#[derive(Component)]
struct GoText;

// Audio markers
#[derive(Component)]
struct BgMusic;

// Resources
#[derive(Resource)]
struct Game {
    phase: Phase,
    timer: f32,
    question: usize,
    score_l: i32,
    score_r: i32,
    picked: Option<Choice>,
    wait: f32,
    session_id: String,
    last_tick: i32,
    hovered_card: Option<Choice>,
    results_shown: bool,
    timeouts: i32,
    streak: i32,
    used_questions: Vec<usize>,
    total_reaction_time: f32,
    answers_count: i32,
    last_reaction: f32,
    tremble: f32,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            phase: Phase::Intro,
            timer: QUESTION_TIME,
            question: 0,
            score_l: 0,
            score_r: 0,
            picked: None,
            wait: 1.5,
            session_id: uuid::Uuid::new_v4().to_string(),
            last_tick: 5,
            hovered_card: None,
            results_shown: false,
            timeouts: 0,
            streak: 0,
            used_questions: vec![0],
            total_reaction_time: 0.0,
            answers_count: 0,
            last_reaction: 5.0,
            tremble: 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
enum Phase {
    #[default]
    Intro,
    Playing,
    Picked,
    Transition,
    UhOh,
    Results,
}

#[derive(Resource)]
struct Questions(Vec<Q>);

struct Q {
    title: &'static str,
    left: &'static str,
    left_em: &'static str,
    right: &'static str,
    right_em: &'static str,
    trait_name: &'static str,
}

impl Default for Questions {
    fn default() -> Self {
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        let mut all = vec![
            Q { title: "There are 2 types of people...", left: "EARLY\nBIRD", left_em: "", right: "NIGHT\nOWL", right_em: "", trait_name: "" },
            Q { title: "When the alarm goes off...", left: "SNOOZE\nx100", left_em: "", right: "UP &\nAT EM", right_em: "", trait_name: "" },
            Q { title: "Your phone battery...", left: "5%\nALWAYS", left_em: "", right: "ALWAYS\n100%", right_em: "", trait_name: "" },
            Q { title: "Texting back takes...", left: "3-5\nDAYS", left_em: "", right: "INSTANT\nREPLY", right_em: "", trait_name: "" },
            Q { title: "Friday night = ...", left: "COUCH\nNETFLIX", left_em: "", right: "OUT\nTILL 4AM", right_em: "", trait_name: "" },
            Q { title: "Lights on or off...", left: "LIGHTS\nON", left_em: "", right: "LIGHTS\nOFF", right_em: "", trait_name: "" },
            Q { title: "Big spoon or...", left: "BIG\nSPOON", left_em: "", right: "LITTLE\nSPOON", right_em: "", trait_name: "" },
            Q { title: "First date energy...", left: "NERVOUS\nWRECK", left_em: "", right: "MAIN\nCHARACTER", right_em: "", trait_name: "" },
            Q { title: "Flirting style be like...", left: "EYE\nCONTACT", left_em: "", right: "JUST\nSAY IT", right_em: "", trait_name: "" },
            Q { title: "When the vibe is off...", left: "GHOST\nTHEM", left_em: "", right: "TALK IT\nOUT", right_em: "", trait_name: "" },
            Q { title: "Music during...", left: "YES\nALWAYS", left_em: "", right: "SILENCE\nIS GOLD", right_em: "", trait_name: "" },
            Q { title: "Morning or night...", left: "SUNRISE\nENERGY", left_em: "", right: "AFTER\nDARK", right_em: "", trait_name: "" },
            Q { title: "They ate your leftovers...", left: "WAR\nCRIME", left_em: "", right: "IT'S JUST\nFOOD", right_em: "", trait_name: "" },
            Q { title: "Netflix and...", left: "ACTUALLY\nWATCH", left_em: "", right: "WHO'S\nWATCHING", right_em: "", trait_name: "" },
            Q { title: "Pet names in public...", left: "BABY\nBABE", left_em: "", right: "FIRST\nNAME", right_em: "", trait_name: "" },
            Q { title: "Thermostat wars...", left: "ARCTIC\nBLAST", left_em: "", right: "SAUNA\nMODE", right_em: "", trait_name: "" },
            Q { title: "Road trip roles...", left: "DJ &\nNAVIGATOR", left_em: "", right: "DRIVER\nONLY", right_em: "", trait_name: "" },
            Q { title: "Saying I love you...", left: "EVERY\n5 MIN", left_em: "", right: "WHEN IT\nMATTERS", right_em: "", trait_name: "" },
            Q { title: "Going to bed angry...", left: "NEVER\nEVER", left_em: "", right: "SLEEP\nON IT", right_em: "", trait_name: "" },
            Q { title: "Drunk behavior...", left: "CLINGY\nAF", left_em: "", right: "SLEEPY\nQUIET", right_em: "", trait_name: "" },
        ];
        all.shuffle(&mut rng);
        Self(all)
    }
}

#[derive(Resource)]
struct GameSounds {
    hover: Handle<AudioSource>,
    click: Handle<AudioSource>,
    tick: Handle<AudioSource>,
    tick_urgent: Handle<AudioSource>,
    whoosh: Handle<AudioSource>,
    result: Handle<AudioSource>,
    select: Handle<AudioSource>,
    go: Handle<AudioSource>,
    card_in: Handle<AudioSource>,
}

#[derive(Resource)]
struct DbPool(Arc<Mutex<Option<MySqlPool>>>);

impl Default for DbPool {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }
}

#[derive(Resource)]
struct TokioRuntime(Runtime);

// Events for audio playback
#[derive(Event)]
struct PlaySoundEvent(SoundType);

#[derive(Event)]
struct SpawnFireworksEvent {
    x: f32,
    intensity: i32,
}

#[derive(Clone, Copy)]
enum SoundType {
    Hover,
    Click,
    Tick,
    TickUrgent,
    Whoosh,
    Result,
    Select,
    Go,
    CardIn,
}

/// Creates a rounded rectangle mesh for cards
fn create_rounded_rect_mesh(width: f32, height: f32, radius: f32) -> Mesh {
    use bevy::render::mesh::{Indices, PrimitiveTopology};
    
    let hw = width / 2.0;
    let hh = height / 2.0;
    let r = radius.min(hw).min(hh); // Clamp radius
    let segments = 8; // Segments per corner
    
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    
    // Center vertex
    positions.push([0.0, 0.0, 0.0]);
    uvs.push([0.5, 0.5]);
    
    // Generate vertices around the rounded rectangle
    let corners = [
        (hw - r, hh - r, 0.0),              // Top-right
        (-hw + r, hh - r, std::f32::consts::FRAC_PI_2),  // Top-left
        (-hw + r, -hh + r, std::f32::consts::PI),        // Bottom-left
        (hw - r, -hh + r, std::f32::consts::PI * 1.5),   // Bottom-right
    ];
    
    for (cx, cy, start_angle) in corners {
        for i in 0..=segments {
            let angle = start_angle + (i as f32 / segments as f32) * std::f32::consts::FRAC_PI_2;
            let x = cx + r * angle.cos();
            let y = cy + r * angle.sin();
            positions.push([x, y, 0.0]);
            uvs.push([(x / width) + 0.5, (y / height) + 0.5]);
        }
    }
    
    // Generate triangle fan indices
    let num_outer = positions.len() as u32 - 1;
    for i in 1..=num_outer {
        let next = if i == num_outer { 1 } else { i + 1 };
        indices.extend_from_slice(&[0, i, next]);
    }
    
    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

fn main() {
    // Load env vars
    let _ = dotenvy::dotenv();

    // Create tokio runtime for async DB operations
    let runtime = Runtime::new().expect("Failed to create Tokio runtime");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "TWO PEOPLE - CHOOSE FAST!".into(),
                mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(BG_COLOR))
        .init_resource::<Game>()
        .init_resource::<Questions>()
        .init_resource::<DbPool>()
        .init_resource::<DbStats>()
        .insert_resource(TokioRuntime(runtime))
        .add_event::<PlaySoundEvent>()
        .add_event::<SpawnFireworksEvent>()
        .add_systems(Startup, (setup, setup_audio, setup_db))
        .add_systems(
            Update,
            (
                intro_tick,
                timer_tick,
                hover_cards,
                click_cards,
                picked_tick,
                transition_tick,
                show_results,
                update_visuals,
                animate_particles,
                animate_bg_shapes,
                animate_fireworks,
                spawn_fireworks,
                animate_pulse,
                screen_shake,
                handle_sound_events,
                handle_replay,
                uhoh_tick,
            ),
        )
        .run();
}

fn setup_audio(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let sounds = GameSounds {
        hover: asset_server.load("sounds/hover.ogg"),
        click: asset_server.load("sounds/click.ogg"),
        tick: asset_server.load("sounds/tick.ogg"),
        tick_urgent: asset_server.load("sounds/tick_urgent.ogg"),
        whoosh: asset_server.load("sounds/whoosh.ogg"),
        result: asset_server.load("sounds/result.ogg"),
        select: asset_server.load("sounds/select.ogg"),
        go: asset_server.load("sounds/go.ogg"),
        card_in: asset_server.load("sounds/card_in.ogg"),
    };
    cmd.insert_resource(sounds);

    cmd.spawn((
        AudioPlayer::new(asset_server.load("sounds/music.ogg")),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::new(1.0),
            ..default()
        },
        BgMusic,
    ));
}

fn setup_db(db_pool: Res<DbPool>, runtime: Res<TokioRuntime>) {
    let database_url = std::env::var("DATABASE_URL").ok();
    let pool_arc = db_pool.0.clone();

    if let Some(url) = database_url {
        info!("Database URL found, connecting...");

        // Spawn async task to connect to database
        runtime.0.spawn(async move {
            match MySqlPoolOptions::new()
                .max_connections(5)
                .connect(&url)
                .await
            {
                Ok(pool) => {
                    // Create table if not exists
                    let create_table = r#"
                        CREATE TABLE IF NOT EXISTS game_scores (
                            id INT AUTO_INCREMENT PRIMARY KEY,
                            session_id VARCHAR(36) NOT NULL,
                            score_left INT NOT NULL,
                            score_right INT NOT NULL,
                            result_type VARCHAR(50) NOT NULL,
                            played_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                        )
                    "#;

                    if let Err(e) = sqlx::query(create_table).execute(&pool).await {
                        warn!("Failed to create table: {}", e);
                    } else {
                        info!("Database connected and table ready");
                    }

                    let mut lock = pool_arc.lock().await;
                    *lock = Some(pool);
                }
                Err(e) => {
                    warn!("Failed to connect to database: {}", e);
                }
            }
        });
    } else {
        warn!("No DATABASE_URL found in environment, scores won't be saved to database");
    }
}

fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    qs: Res<Questions>,
) {
    // Camera
    cmd.spawn(Camera2d);

    // Title (question text - shown above cards)
    cmd.spawn((
        Text2d::new("TWO PEOPLE"),
        TextFont {
            font_size: 48.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 250.0, 10.0),
        TitleText,
        Pulse { speed: 3.0 },
    ));

    // "CHOOSE FAST!" subtitle
    cmd.spawn((
        Text2d::new("CHOOSE FAST!"),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(TEXT_YELLOW),
        Transform::from_xyz(0.0, 200.0, 10.0),
        HurryText,
    ));

    // Timer (top of screen)
    cmd.spawn((
        Text2d::new("5"),
        TextFont {
            font_size: 56.0,
            ..default()
        },
        TextColor(TIMER_NORMAL),
        Transform::from_xyz(0.0, 320.0, 10.0),
        Visibility::Hidden,
        TimerDisplay,
    ));

    // Result
    cmd.spawn((
        Text2d::new(""),
        TextFont {
            font_size: 64.0,
            ..default()
        },
        TextColor(RESULT_GREEN),
        Transform::from_xyz(0.0, 0.0, 10.0),
        Visibility::Hidden,
        ResultDisplay,
    ));

    let q = &qs.0[0];
    let card_mesh = meshes.add(create_rounded_rect_mesh(CARD_W, CARD_H, 25.0));

    // Left card
    let lx = -CARD_GAP / 2.0;
    cmd.spawn((
        Mesh2d(card_mesh.clone()),
        MeshMaterial2d(mats.add(ColorMaterial::from(CARD_LEFT))),
        Transform::from_xyz(lx, -20.0, 0.0).with_scale(Vec3::ZERO),
        Visibility::Hidden,
        Card {
            choice: Choice::Left,
            base_y: -20.0,
        },
    ));
    cmd.spawn((
        Text2d::new(q.left.to_string()),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(lx, -20.0, 1.0).with_scale(Vec3::ZERO),
        Visibility::Hidden,
        CardLabel {
            choice: Choice::Left,
        },
    ));

    // Right card
    let rx = CARD_GAP / 2.0;
    cmd.spawn((
        Mesh2d(card_mesh.clone()),
        MeshMaterial2d(mats.add(ColorMaterial::from(CARD_RIGHT))),
        Transform::from_xyz(rx, -20.0, 0.0).with_scale(Vec3::ZERO),
        Visibility::Hidden,
        Card {
            choice: Choice::Right,
            base_y: -20.0,
        },
    ));
    cmd.spawn((
        Text2d::new(q.right.to_string()),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(rx, -20.0, 1.0).with_scale(Vec3::ZERO),
        Visibility::Hidden,
        CardLabel {
            choice: Choice::Right,
        },
    ));

    let mut rng = rand::rng();
    
    for i in 0..15 {
        let size = rng.random_range(200.0..700.0);
        let x = rng.random_range(-900.0..900.0);
        let y = rng.random_range(-500.0..500.0);
        let a = rng.random_range(0.02..0.06);
        let hue = (i as f32 / 15.0) * 360.0;
        let c = Color::hsla(hue, 0.6, 0.5, a);
        let sides = [3, 4, 5, 6, 8][rng.random_range(0..5)];
        let mesh = meshes.add(RegularPolygon::new(size, sides));
        cmd.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(mats.add(ColorMaterial::from(c))),
            Transform::from_xyz(x, y, -10.0),
            BgShape {
                spin_speed: rng.random_range(-0.15..0.15),
                pulse_speed: rng.random_range(0.3..0.8),
                phase: rng.random_range(0.0..std::f32::consts::TAU),
            },
        ));
    }

    for _ in 0..100 {
        let x = rng.random_range(-WINDOW_WIDTH / 2.0..WINDOW_WIDTH / 2.0);
        let y = rng.random_range(-WINDOW_HEIGHT / 2.0..WINDOW_HEIGHT / 2.0);
        let s = rng.random_range(8.0..40.0);
        let a = rng.random_range(0.03..0.12);
        let hue = rng.random_range(0.0..360.0);
        let c = Color::hsla(hue, 0.5, 0.5, a);
        let mesh = meshes.add(Circle::new(s));
        cmd.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(mats.add(ColorMaterial::from(c))),
            Transform::from_xyz(x, y, -5.0),
            Particle {
                vel: Vec2::new(
                    rng.random_range(-15.0..15.0),
                    rng.random_range(8.0..25.0),
                ),
                phase: rng.random_range(0.0..std::f32::consts::TAU),
                spin: rng.random_range(-0.3..0.3),
            },
        ));
    }

    cmd.spawn((
        Text2d::new("Click a card to choose!"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        Transform::from_xyz(0.0, -320.0, 10.0),
        ReplayInstruction,
    ));

    cmd.spawn((
        Text2d::new(""),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 0.5, 0.9)),
        Transform::from_xyz(0.0, -80.0, 10.0),
        Visibility::Hidden,
        StatsDisplay,
    ));

    for offset in [(6.0, -6.0, 22.0), (4.0, -4.0, 23.0), (2.0, -2.0, 24.0)] {
        cmd.spawn((
            Text2d::new("UH OH! TOO SLOW!"),
            TextFont {
                font_size: 100.0,
                ..default()
            },
            TextColor(Color::BLACK),
            Transform::from_xyz(offset.0, offset.1, offset.2),
            Visibility::Hidden,
            UhOhText,
        ));
    }
    cmd.spawn((
        Text2d::new("UH OH! TOO SLOW!"),
        TextFont {
            font_size: 100.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.0)),
        Transform::from_xyz(0.0, 0.0, 25.0),
        Visibility::Hidden,
        UhOhText,
    ));

    // "GO!" text for transitions (hidden initially)
    cmd.spawn((
        Text2d::new("GO!"),
        TextFont {
            font_size: 120.0,
            ..default()
        },
        TextColor(TEXT_YELLOW),
        Transform::from_xyz(0.0, 0.0, 20.0),
        Visibility::Hidden,
        GoText,
    ));
}

fn handle_sound_events(
    mut cmd: Commands,
    mut events: EventReader<PlaySoundEvent>,
    sounds: Option<Res<GameSounds>>,
) {
    let Some(sounds) = sounds else { return };

    for event in events.read() {
        let source = match event.0 {
            SoundType::Hover => sounds.hover.clone(),
            SoundType::Click => sounds.click.clone(),
            SoundType::Tick => sounds.tick.clone(),
            SoundType::TickUrgent => sounds.tick_urgent.clone(),
            SoundType::Whoosh => sounds.whoosh.clone(),
            SoundType::Result => sounds.result.clone(),
            SoundType::Select => sounds.select.clone(),
            SoundType::Go => sounds.go.clone(),
            SoundType::CardIn => sounds.card_in.clone(),
        };

        // Spawn audio with PlaybackSettings
        cmd.spawn((
            AudioPlayer::new(source),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::new(0.5),
                ..default()
            },
        ));
    }
}

fn intro_tick(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut title: Query<&mut Visibility, With<TitleText>>,
    mut hurry: Query<&mut Visibility, (With<HurryText>, Without<TitleText>)>,
    mut cards: Query<
        (&mut Visibility, &mut Transform),
        (With<Card>, Without<TitleText>, Without<HurryText>),
    >,
    mut labels: Query<
        (&mut Visibility, &mut Transform),
        (
            With<CardLabel>,
            Without<Card>,
            Without<TitleText>,
            Without<HurryText>,
        ),
    >,
    mut timer_vis: Query<
        &mut Visibility,
        (
            With<TimerDisplay>,
            Without<TitleText>,
            Without<HurryText>,
            Without<Card>,
            Without<CardLabel>,
        ),
    >,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    if game.phase != Phase::Intro {
        return;
    }

    game.wait -= time.delta_secs();
    if game.wait <= 0.0 {
        game.phase = Phase::Playing;
        game.timer = QUESTION_TIME;
        game.last_tick = 5;

        // Play whoosh sound when cards appear
        sound_events.send(PlaySoundEvent(SoundType::Whoosh));

        // Hide intro
        for mut v in title.iter_mut() {
            *v = Visibility::Hidden;
        }
        for mut v in hurry.iter_mut() {
            *v = Visibility::Hidden;
        }

        // Show cards + timer
        for (mut v, mut t) in cards.iter_mut() {
            *v = Visibility::Visible;
            t.scale = Vec3::ONE;
        }
        for (mut v, mut t) in labels.iter_mut() {
            *v = Visibility::Visible;
            t.scale = Vec3::ONE;
        }
        for mut v in timer_vis.iter_mut() {
            *v = Visibility::Visible;
        }
    }
}

fn timer_tick(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    if game.phase != Phase::Playing {
        return;
    }
    game.timer -= time.delta_secs();

    let current_sec = game.timer.ceil() as i32;
    if current_sec < game.last_tick && current_sec >= 0 {
        game.last_tick = current_sec;
        if current_sec <= 2 {
            sound_events.send(PlaySoundEvent(SoundType::TickUrgent));
        } else {
            sound_events.send(PlaySoundEvent(SoundType::Tick));
        }
    }

    if game.timer <= 0.0 {
        game.timeouts += 1;
        game.streak = 0;
        
        if game.timeouts >= 3 {
            game.phase = Phase::Results;
            sound_events.send(PlaySoundEvent(SoundType::Result));
        } else {
            game.phase = Phase::UhOh;
            game.wait = 1.2;
            sound_events.send(PlaySoundEvent(SoundType::Whoosh));
        }
    }
}

fn hover_cards(
    windows: Query<&Window, With<PrimaryWindow>>,
    cam: Query<(&Camera, &GlobalTransform)>,
    mut cards: Query<(&Card, &mut Transform, &GlobalTransform)>,
    mut labels: Query<(&CardLabel, &mut Transform), Without<Card>>,
    mut game: ResMut<Game>,
    time: Res<Time>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    if game.phase != Phase::Playing {
        return;
    }

    let Ok(win) = windows.get_single() else {
        return;
    };
    let Ok((camera, cam_t)) = cam.get_single() else {
        return;
    };
    let Some(cursor) = win.cursor_position() else {
        return;
    };
    let Some(world) = camera.viewport_to_world_2d(cam_t, cursor).ok() else {
        return;
    };

    let mut new_hover: Option<Choice> = None;

    for (card, mut t, gt) in cards.iter_mut() {
        let pos = gt.translation().truncate();
        let hovered = world.x >= pos.x - CARD_W / 2.0
            && world.x <= pos.x + CARD_W / 2.0
            && world.y >= pos.y - CARD_H / 2.0
            && world.y <= pos.y + CARD_H / 2.0;

        if hovered {
            new_hover = Some(card.choice);
        }

        let t_secs = time.elapsed_secs();
        let after_warmup = (game.answers_count as f32 - 20.0).max(0.0);
        let progress = after_warmup / 10.0;
        let chaos = (progress + game.tremble * 2.0).min(3.0);
        let bounce_intensity = 1.0 + chaos * 4.0;
        let bounce_speed = 2.0 + chaos * 10.0;
        
        let base_scale = if hovered { HOVER_SCALE } else { 1.0 };
        let pulse = 1.0 + (t_secs * (10.0 + chaos * 20.0)).sin().abs() * 0.1 * chaos;
        t.scale = t.scale.lerp(Vec3::splat(base_scale * pulse), 15.0 * time.delta_secs());

        let phase_offset = if card.choice == Choice::Left { 0.0 } else { std::f32::consts::PI };
        let bob = (t_secs * bounce_speed + phase_offset).sin() * (6.0 + 4.0 * chaos);
        let side_bob = (t_secs * bounce_speed * 0.7 + phase_offset).cos() * 5.0 * chaos;
        
        let tremble_x = (t_secs * 50.0 + phase_offset).sin() * 8.0 * game.tremble;
        let tremble_y = (t_secs * 55.0).cos() * 6.0 * game.tremble;
        
        let urgency_factor = ((game.answers_count as f32 - 20.0).max(0.0) / 15.0).min(1.5);
        let time_elapsed = 1.0 - (game.timer / QUESTION_TIME);
        let urgency_shake = time_elapsed * time_elapsed * urgency_factor * 12.0;
        let uh_oh_x = (t_secs * 45.0 + phase_offset).sin() * urgency_shake;
        let uh_oh_y = (t_secs * 52.0).cos() * urgency_shake * 0.7;
        
        let spin = (t_secs * bounce_speed * 0.5 + phase_offset).cos() * 0.15 * chaos;
        let panic_spin = (t_secs * 40.0).sin() * 0.1 * game.tremble;
        let uh_oh_spin = (t_secs * 38.0).sin() * 0.04 * time_elapsed * urgency_factor;
        
        let base_x = if card.choice == Choice::Left { -CARD_GAP / 2.0 } else { CARD_GAP / 2.0 };
        t.translation.y = card.base_y + bob + tremble_y + uh_oh_y;
        t.translation.x = base_x + side_bob + tremble_x + uh_oh_x;
        t.rotation = Quat::from_rotation_z(spin + panic_spin + uh_oh_spin);
    }

    // Detect hover change and play sound
    if new_hover != game.hovered_card {
        if new_hover.is_some() {
            sound_events.send(PlaySoundEvent(SoundType::Hover));
        }
        game.hovered_card = new_hover;
    }

    // Sync label positions with cards
    for (label, mut lt) in labels.iter_mut() {
        for (card, ct, _) in cards.iter() {
            if card.choice == label.choice {
                lt.translation = ct.translation + Vec3::Z;
                lt.scale = ct.scale;
            }
        }
    }
}

fn click_cards(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cam: Query<(&Camera, &GlobalTransform)>,
    mut game: ResMut<Game>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut firework_events: EventWriter<SpawnFireworksEvent>,
) {
    if game.phase != Phase::Playing || !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(win) = windows.get_single() else {
        return;
    };
    let Ok((camera, cam_t)) = cam.get_single() else {
        return;
    };
    let Some(cursor) = win.cursor_position() else {
        return;
    };
    let Some(world) = camera.viewport_to_world_2d(cam_t, cursor).ok() else {
        return;
    };

    let choice = if world.x < 0.0 { Choice::Left } else { Choice::Right };
    
    let reaction_time = QUESTION_TIME - game.timer;
    game.total_reaction_time += reaction_time;
    game.answers_count += 1;
    game.last_reaction = reaction_time;
    
    game.tremble = match reaction_time {
        t if t < 0.5 => (game.tremble + 0.05).min(1.5),
        t if t < 1.0 => (game.tremble + 0.02).min(1.5),
        t if t < 2.0 => (game.tremble + 0.008).min(1.5),
        _ => (game.tremble - 0.1).max(0.0),
    };
    
    let avg_reaction = game.total_reaction_time / game.answers_count as f32;
    let speed_bonus = ((QUESTION_TIME - avg_reaction) / QUESTION_TIME * 10.0) as i32;
    
    sound_events.send(PlaySoundEvent(SoundType::Select));
    game.picked = Some(choice);
    game.timeouts = 0;
    game.streak += 1;
    
    let intensity = game.streak + speed_bonus.max(0) + (game.tremble * 5.0) as i32;
    firework_events.send(SpawnFireworksEvent { x: 0.0, intensity });
    
    match choice {
        Choice::Left => game.score_l += 1,
        Choice::Right => game.score_r += 1,
    };
    game.phase = Phase::Picked;
    game.wait = 0.6;
}

fn picked_tick(
    time: Res<Time>,
    mut game: ResMut<Game>,
    qs: Res<Questions>,
    mut cards: Query<(&Card, &mut Transform), Without<CardLabel>>,
    mut labels: Query<(&CardLabel, &mut Transform), Without<Card>>,
    mut title: Query<&mut Visibility, With<TitleText>>,
    mut timer_vis: Query<
        &mut Visibility,
        (
            With<TimerDisplay>,
            Without<TitleText>,
        ),
    >,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    if game.phase != Phase::Picked {
        return;
    }

    let lerp_speed = 12.0 * time.delta_secs();

    for (card, mut t) in cards.iter_mut() {
        if Some(card.choice) == game.picked {
            t.translation.x = t.translation.x * (1.0 - lerp_speed) + 0.0 * lerp_speed;
            t.scale = t.scale.lerp(Vec3::splat(1.3), lerp_speed);
        } else {
            let fly_dir = if card.choice == Choice::Left { -1.0 } else { 1.0 };
            t.translation.x += fly_dir * 1500.0 * time.delta_secs();
            t.scale = t.scale.lerp(Vec3::splat(0.5), lerp_speed * 2.0);
        }
    }

    for (label, mut lt) in labels.iter_mut() {
        for (card, ct) in cards.iter() {
            if card.choice == label.choice {
                lt.translation.x = ct.translation.x;
                lt.translation.y = ct.translation.y;
                lt.scale = ct.scale;
            }
        }
    }

    for mut v in title.iter_mut() {
        *v = Visibility::Hidden;
    }
    for mut v in timer_vis.iter_mut() {
        *v = Visibility::Hidden;
    }

    game.wait -= time.delta_secs();
    if game.wait <= 0.0 {
        game.phase = Phase::Transition;
        let warmup = (20 - game.answers_count).max(0) as f32 / 20.0;
        let speed_factor = if game.answers_count < 20 {
            0.5
        } else if game.last_reaction < 1.0 {
            0.15
        } else {
            0.3
        };
        game.wait = speed_factor;
        sound_events.send(PlaySoundEvent(SoundType::Go));
    }
}

fn transition_tick(
    time: Res<Time>,
    mut game: ResMut<Game>,
    qs: Res<Questions>,
    mut cards: Query<(&Card, &mut Transform, &mut Visibility), Without<CardLabel>>,
    mut labels: Query<(&CardLabel, &mut Text2d, &mut Transform, &mut Visibility), Without<Card>>,
    mut title: Query<(&mut Text2d, &mut Visibility), (With<TitleText>, Without<CardLabel>, Without<Card>)>,
    mut timer_vis: Query<&mut Visibility, (With<TimerDisplay>, Without<TitleText>, Without<CardLabel>, Without<Card>)>,
    mut go_text: Query<(&mut Visibility, &mut Transform), (With<GoText>, Without<Card>, Without<CardLabel>, Without<TitleText>, Without<TimerDisplay>)>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    if game.phase != Phase::Transition {
        for (mut v, _) in go_text.iter_mut() {
            *v = Visibility::Hidden;
        }
        return;
    }

    for (mut v, mut t) in go_text.iter_mut() {
        *v = Visibility::Visible;
        let pulse = 1.0 + (game.wait * 20.0).sin().abs() * 0.3;
        t.scale = Vec3::splat(pulse);
    }

    game.wait -= time.delta_secs();
    if game.wait <= 0.0 {
        for (mut v, _) in go_text.iter_mut() {
            *v = Visibility::Hidden;
        }

        let mut rng = rand::rng();
        let available: Vec<usize> = (0..qs.0.len())
            .filter(|i| !game.used_questions.contains(i))
            .collect();
        
        if available.is_empty() {
            game.phase = Phase::Results;
            sound_events.send(PlaySoundEvent(SoundType::Result));
            return;
        }
        
        let new_question = available[rng.random_range(0..available.len())];
        game.question = new_question;
        game.used_questions.push(new_question);

        let q = &qs.0[game.question];
        game.phase = Phase::Playing;
        
        let time_pressure = if game.answers_count < 20 {
            QUESTION_TIME
        } else {
            let avg_reaction = game.total_reaction_time / game.answers_count as f32;
            (QUESTION_TIME - (avg_reaction * 0.8)).max(1.5)
        };
        game.timer = time_pressure;
        game.last_tick = time_pressure.ceil() as i32;
        game.picked = None;

        sound_events.send(PlaySoundEvent(SoundType::CardIn));

        for (card, mut t, mut v) in cards.iter_mut() {
            *v = Visibility::Visible;
            t.scale = Vec3::ONE;
            t.translation.x = if card.choice == Choice::Left {
                -CARD_GAP / 2.0
            } else {
                CARD_GAP / 2.0
            };
        }

        for (lbl, mut txt, mut t, mut v) in labels.iter_mut() {
            *v = Visibility::Visible;
            t.scale = Vec3::ONE;
            match lbl.choice {
                Choice::Left => {
                    txt.0 = q.left.to_string();
                }
                Choice::Right => {
                    txt.0 = q.right.to_string();
                    t.translation.x = CARD_GAP / 2.0;
                }
            }
        }

        for (mut txt, mut vis) in title.iter_mut() {
            txt.0 = q.title.into();
            *vis = Visibility::Visible;
        }

        for mut v in timer_vis.iter_mut() {
            *v = Visibility::Visible;
        }
    }
}

fn show_results(
    mut game: ResMut<Game>,
    mut db_stats: ResMut<DbStats>,
    mut cards: Query<&mut Transform, (With<Card>, Without<CardLabel>)>,
    mut labels: Query<&mut Transform, (With<CardLabel>, Without<Card>)>,
    mut title: Query<(&mut Text2d, &mut Visibility), (With<TitleText>, Without<CardLabel>, Without<Card>)>,
    mut result: Query<(&mut Text2d, &mut Visibility), (With<ResultDisplay>, Without<TitleText>, Without<CardLabel>, Without<Card>)>,
    mut stats_display: Query<(&mut Text2d, &mut Visibility), (With<StatsDisplay>, Without<ResultDisplay>, Without<TitleText>, Without<CardLabel>, Without<Card>)>,
    mut timer_vis: Query<&mut Visibility, (With<TimerDisplay>, Without<TitleText>, Without<ResultDisplay>, Without<CardLabel>, Without<Card>, Without<StatsDisplay>)>,
    mut replay_text: Query<&mut Text2d, (With<ReplayInstruction>, Without<TitleText>, Without<ResultDisplay>, Without<CardLabel>, Without<Card>, Without<StatsDisplay>)>,
    db_pool: Res<DbPool>,
    runtime: Res<TokioRuntime>,
) {
    if game.phase != Phase::Results || game.results_shown {
        return;
    }
    game.results_shown = true;

    let (res, result_type) = if game.score_l > game.score_r {
        ("You're a CHAOTIC GREMLIN!", "chaotic_gremlin")
    } else if game.score_r > game.score_l {
        ("You're a FUNCTIONING ADULT!", "functioning_adult")
    } else {
        ("You're PERFECTLY BALANCED!", "perfectly_balanced")
    };

    let pool_arc = db_pool.0.clone();
    let session_id = game.session_id.clone();
    let score_l = game.score_l;
    let score_r = game.score_r;
    let result_type_owned = result_type.to_string();
    let total = (score_l + score_r) as f64;
    let my_left_pct = if total > 0.0 { (score_l as f64 / total) * 100.0 } else { 50.0 };

    runtime.0.spawn(async move {
        let lock = pool_arc.lock().await;
        if let Some(ref pool) = *lock {
            let query = "INSERT INTO game_scores (session_id, score_left, score_right, result_type) VALUES (?, ?, ?, ?)";
            let _ = sqlx::query(query)
                .bind(&session_id)
                .bind(score_l)
                .bind(score_r)
                .bind(&result_type_owned)
                .execute(pool)
                .await;
        }
    });

    if !db_stats.loaded && score_l + score_r >= 3 {
        let pool_arc2 = db_pool.0.clone();
        let stats_ptr = std::ptr::addr_of_mut!(*db_stats) as usize;
        
        runtime.0.spawn(async move {
            let lock = pool_arc2.lock().await;
            if let Some(ref pool) = *lock {
                let row: Option<(i64, f64)> = sqlx::query_as(
                    "SELECT COUNT(*), AVG(score_left * 100.0 / (score_left + score_right)) FROM game_scores WHERE score_left + score_right >= 3"
                )
                .fetch_optional(pool)
                .await
                .ok()
                .flatten();
                
                if let Some((count, avg)) = row {
                    if count >= 3 {
                        unsafe {
                            let stats = &mut *(stats_ptr as *mut DbStats);
                            stats.total_players = count;
                            stats.avg_left_pct = avg;
                            stats.loaded = true;
                        }
                    }
                }
            }
        });
    }

    let stats_text = if db_stats.loaded && db_stats.total_players >= 3 {
        format!(
            "{} players | Avg: {:.0}% chaotic vs {:.0}% adult\nYou: {:.0}% chaotic",
            db_stats.total_players,
            db_stats.avg_left_pct,
            100.0 - db_stats.avg_left_pct,
            my_left_pct
        )
    } else {
        String::new()
    };

    for (mut txt, mut vis) in result.iter_mut() {
        txt.0 = res.into();
        *vis = Visibility::Visible;
    }
    for (mut txt, mut vis) in stats_display.iter_mut() {
        if !stats_text.is_empty() {
            txt.0 = stats_text.clone();
            *vis = Visibility::Visible;
        }
    }
    for mut t in cards.iter_mut() {
        t.scale = Vec3::ZERO;
    }
    for mut t in labels.iter_mut() {
        t.scale = Vec3::ZERO;
    }
    for mut v in timer_vis.iter_mut() {
        *v = Visibility::Hidden;
    }
    for (mut txt, mut vis) in title.iter_mut() {
        txt.0 = "Press R to play again!".into();
        *vis = Visibility::Visible;
    }
    for mut txt in replay_text.iter_mut() {
        txt.0 = "Press R to restart".into();
    }
}

fn handle_replay(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
    qs: Res<Questions>,
    mut cards: Query<(&Card, &mut Visibility, &mut Transform), Without<CardLabel>>,
    mut labels: Query<(&CardLabel, &mut Text2d, &mut Visibility, &mut Transform), Without<Card>>,
    mut title: Query<(&mut Text2d, &mut Visibility), (With<TitleText>, Without<CardLabel>, Without<Card>)>,
    mut result: Query<&mut Visibility, (With<ResultDisplay>, Without<TitleText>, Without<CardLabel>, Without<Card>)>,
    mut stats_vis: Query<&mut Visibility, (With<StatsDisplay>, Without<ResultDisplay>, Without<TitleText>, Without<CardLabel>, Without<Card>)>,
    mut timer_vis: Query<&mut Visibility, (With<TimerDisplay>, Without<TitleText>, Without<ResultDisplay>, Without<CardLabel>, Without<Card>, Without<StatsDisplay>)>,
    mut replay_text: Query<&mut Text2d, (With<ReplayInstruction>, Without<TitleText>, Without<ResultDisplay>, Without<CardLabel>, Without<Card>, Without<StatsDisplay>)>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    if game.phase != Phase::Results {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        game.phase = Phase::Playing;
        game.timer = QUESTION_TIME;
        game.question = 0;
        game.score_l = 0;
        game.score_r = 0;
        game.picked = None;
        game.wait = 0.0;
        game.session_id = uuid::Uuid::new_v4().to_string();
        game.last_tick = 5;
        game.hovered_card = None;
        game.results_shown = false;
        game.timeouts = 0;
        game.streak = 0;
        game.used_questions = vec![0];
        game.total_reaction_time = 0.0;
        game.answers_count = 0;
        game.last_reaction = 5.0;
        game.tremble = 0.0;

        sound_events.send(PlaySoundEvent(SoundType::CardIn));

        let q = &qs.0[0];

        for (card, mut vis, mut t) in cards.iter_mut() {
            *vis = Visibility::Visible;
            t.scale = Vec3::ONE;
            // Reset position
            t.translation.x = if card.choice == Choice::Left {
                -CARD_GAP / 2.0
            } else {
                CARD_GAP / 2.0
            };
        }

        for (lbl, mut txt, mut vis, mut t) in labels.iter_mut() {
            *vis = Visibility::Visible;
            t.scale = Vec3::ONE;
            match lbl.choice {
                Choice::Left => txt.0 = q.left.to_string(),
                Choice::Right => txt.0 = q.right.to_string(),
            }
        }

        for (mut txt, mut vis) in title.iter_mut() {
            txt.0 = q.title.into();
            *vis = Visibility::Visible;
        }

        for mut vis in result.iter_mut() {
            *vis = Visibility::Hidden;
        }

        for mut vis in stats_vis.iter_mut() {
            *vis = Visibility::Hidden;
        }

        for mut vis in timer_vis.iter_mut() {
            *vis = Visibility::Visible;
        }

        for mut txt in replay_text.iter_mut() {
            txt.0 = "Click a card to choose!".into();
        }

        info!("Game restarted with new session: {}", game.session_id);
    }
}

fn update_visuals(
    time: Res<Time>,
    game: Res<Game>,
    mut timer_q: Query<(&mut Text2d, &mut TextColor, &mut Transform), With<TimerDisplay>>,
    mut title_q: Query<(&mut Visibility, &mut Transform), (With<TitleText>, Without<TimerDisplay>)>,
) {
    if game.phase == Phase::Playing {
        let secs = game.timer.ceil() as i32;
        let frac = game.timer.fract();
        let intensity = ((game.answers_count as f32 - 3.0).max(0.0) / 10.0).min(2.0);
        let t_secs = time.elapsed_secs();
        
        for (mut txt, mut col, mut t) in timer_q.iter_mut() {
            txt.0 = format!("{}", secs.max(0));

            let bam = 1.0 - frac;
            let base_scale = 0.5 + bam * (1.5 + intensity * 1.5);
            let shake = if intensity > 0.5 {
                (t_secs * (20.0 + intensity * 40.0)).sin() * 0.1 * intensity
            } else { 0.0 };
            let wobble_rot = (t_secs * (10.0 + intensity * 30.0)).cos() * 0.05 * intensity;
            
            if game.timer <= HURRY_TIME {
                col.0 = TIMER_HURRY;
                let panic = (t_secs * 50.0).sin() * 0.2 * intensity;
                t.scale = Vec3::splat(base_scale * (1.3 + intensity * 0.5) + panic);
                t.rotation = Quat::from_rotation_z(wobble_rot * 2.0);
            } else {
                col.0 = TIMER_NORMAL;
                t.scale = Vec3::splat(base_scale);
                t.rotation = Quat::from_rotation_z(wobble_rot);
            }
        }

        for (mut v, mut t) in title_q.iter_mut() {
            *v = Visibility::Visible;
            let bounce = (t_secs * (3.0 + intensity * 3.0)).sin() * 3.0 * (1.0 + intensity);
            let wobble = (t_secs * 5.0).cos() * 0.02 * intensity;
            t.translation.y = 280.0 + bounce;
            t.rotation = Quat::from_rotation_z(wobble);
        }
    }
}

fn animate_particles(time: Res<Time>, mut particles: Query<(&mut Transform, &Particle)>) {
    let mut rng = rand::rng();
    let t_secs = time.elapsed_secs();
    
    for (mut t, p) in particles.iter_mut() {
        t.translation.x += p.vel.x * time.delta_secs();
        t.translation.y += p.vel.y * time.delta_secs();
        t.rotation = Quat::from_rotation_z(t_secs * p.spin + p.phase);
        let wobble = (t_secs * 0.5 + p.phase).sin() * 0.15;
        t.scale = Vec3::splat(1.0 + wobble);

        if t.translation.y > WINDOW_HEIGHT / 2.0 + 60.0 {
            t.translation.y = -WINDOW_HEIGHT / 2.0 - 60.0;
            t.translation.x = rng.random_range(-WINDOW_WIDTH / 2.0..WINDOW_WIDTH / 2.0);
        }
        if t.translation.x > WINDOW_WIDTH / 2.0 + 60.0 {
            t.translation.x = -WINDOW_WIDTH / 2.0 - 60.0;
        }
        if t.translation.x < -WINDOW_WIDTH / 2.0 - 60.0 {
            t.translation.x = WINDOW_WIDTH / 2.0 + 60.0;
        }
    }
}

fn animate_bg_shapes(time: Res<Time>, mut shapes: Query<(&mut Transform, &BgShape)>) {
    let t_secs = time.elapsed_secs();
    
    for (mut t, s) in shapes.iter_mut() {
        t.rotation = Quat::from_rotation_z(t_secs * s.spin_speed + s.phase);
        let pulse = 1.0 + (t_secs * s.pulse_speed + s.phase).sin() * 0.1;
        t.scale = Vec3::splat(pulse);
    }
}

fn animate_pulse(time: Res<Time>, mut q: Query<(&mut Transform, &Pulse)>) {
    for (mut t, p) in q.iter_mut() {
        let s = 1.0 + (time.elapsed_secs() * p.speed).sin() * 0.05;
        t.scale = Vec3::splat(s);
    }
}

fn screen_shake(game: Res<Game>, mut cam: Query<&mut Transform, With<Camera2d>>) {
    let mut rng = rand::rng();
    let shake_intensity = (game.streak as f32 * 0.5).min(8.0);
    for mut t in cam.iter_mut() {
        if game.phase == Phase::Picked && game.wait > 0.4 {
            t.translation.x = rng.random_range(-shake_intensity..shake_intensity);
            t.translation.y = rng.random_range(-shake_intensity..shake_intensity);
        } else {
            t.translation.x *= 0.85;
            t.translation.y *= 0.85;
        }
    }
}

fn spawn_fireworks(
    mut cmd: Commands,
    mut events: EventReader<SpawnFireworksEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::rng();
    
    for event in events.read() {
        let base_count = 20 + (event.intensity as usize * 15).min(200);
        let speed_mult = 1.0 + (event.intensity as f32 * 0.2).min(3.0);
        
        for _ in 0..base_count {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let speed = rng.random_range(200.0..800.0) * speed_mult;
            let size = rng.random_range(4.0..15.0);
            let hue = rng.random_range(0.0..360.0);
            let life = rng.random_range(0.5..1.2);
            
            let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);
            let c = Color::hsla(hue, 1.0, 0.6, 1.0);
            
            let mesh = if rng.random_bool(0.5) {
                meshes.add(Circle::new(size))
            } else {
                meshes.add(RegularPolygon::new(size, rng.random_range(3..7)))
            };
            
            cmd.spawn((
                Mesh2d(mesh),
                MeshMaterial2d(mats.add(ColorMaterial::from(c))),
                Transform::from_xyz(event.x, 0.0, 15.0),
                Firework {
                    vel,
                    life,
                    max_life: life,
                    color_hue: hue,
                },
            ));
        }
        
        for _ in 0..(event.intensity as usize * 5).min(50) {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let speed = rng.random_range(100.0..400.0) * speed_mult;
            let size = rng.random_range(8.0..25.0);
            let hue = rng.random_range(30.0..60.0);
            let life = rng.random_range(0.8..1.5);
            
            let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed + 100.0);
            let c = Color::hsla(hue, 1.0, 0.7, 1.0);
            
            cmd.spawn((
                Mesh2d(meshes.add(RegularPolygon::new(size, 5))),
                MeshMaterial2d(mats.add(ColorMaterial::from(c))),
                Transform::from_xyz(event.x, 0.0, 16.0),
                Firework {
                    vel,
                    life,
                    max_life: life,
                    color_hue: hue,
                },
            ));
        }
    }
}

fn animate_fireworks(
    mut cmd: Commands,
    time: Res<Time>,
    mut fireworks: Query<(Entity, &mut Transform, &mut Firework)>,
) {
    let dt = time.delta_secs();
    let gravity = -600.0;
    
    for (entity, mut t, mut fw) in fireworks.iter_mut() {
        fw.vel.y += gravity * dt;
        t.translation.x += fw.vel.x * dt;
        t.translation.y += fw.vel.y * dt;
        
        let life_pct = fw.life / fw.max_life;
        t.scale = Vec3::splat(life_pct.max(0.1));
        t.rotation = Quat::from_rotation_z(time.elapsed_secs() * 5.0);
        
        fw.life -= dt;
        if fw.life <= 0.0 {
            cmd.entity(entity).despawn();
        }
    }
}

fn uhoh_tick(
    time: Res<Time>,
    mut game: ResMut<Game>,
    qs: Res<Questions>,
    mut uhoh_text: Query<(&mut Visibility, &mut Transform), (With<UhOhText>, Without<Card>, Without<CardLabel>)>,
    mut cards: Query<(&Card, &mut Transform, &mut Visibility), Without<CardLabel>>,
    mut labels: Query<(&CardLabel, &mut Text2d, &mut Transform, &mut Visibility), Without<Card>>,
    mut title: Query<&mut Visibility, (With<TitleText>, Without<Card>, Without<CardLabel>, Without<UhOhText>)>,
    mut timer_vis: Query<&mut Visibility, (With<TimerDisplay>, Without<TitleText>, Without<Card>, Without<CardLabel>, Without<UhOhText>)>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    if game.phase != Phase::UhOh {
        for (mut vis, _) in uhoh_text.iter_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }

    let pulse = (time.elapsed_secs() * 10.0).sin() * 0.1 + 1.0;
    for (mut vis, mut t) in uhoh_text.iter_mut() {
        *vis = Visibility::Visible;
        t.scale = Vec3::splat(pulse);
    }

    for mut v in title.iter_mut() {
        *v = Visibility::Hidden;
    }
    for mut v in timer_vis.iter_mut() {
        *v = Visibility::Hidden;
    }

    game.wait -= time.delta_secs();
    if game.wait <= 0.0 {
        game.question += 1;
        if game.question >= qs.0.len() {
            game.question = 0;
        }

        let q = &qs.0[game.question];
        game.timer = QUESTION_TIME;
        game.last_tick = 5;
        game.picked = None;
        game.phase = Phase::Playing;

        for (card, mut t, mut vis) in cards.iter_mut() {
            *vis = Visibility::Visible;
            t.scale = Vec3::ONE;
            t.translation.x = if card.choice == Choice::Left {
                -CARD_GAP / 2.0
            } else {
                CARD_GAP / 2.0
            };
        }

        for (lbl, mut txt, mut t, mut vis) in labels.iter_mut() {
            *vis = Visibility::Visible;
            t.scale = Vec3::ONE;
            match lbl.choice {
                Choice::Left => {
                    txt.0 = q.left.to_string();
                    t.translation.x = -CARD_GAP / 2.0;
                }
                Choice::Right => {
                    txt.0 = q.right.to_string();
                    t.translation.x = CARD_GAP / 2.0;
                }
            }
        }

        sound_events.send(PlaySoundEvent(SoundType::CardIn));
    }
}
