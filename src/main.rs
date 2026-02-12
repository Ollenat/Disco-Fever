use std::fmt::Display;

use bevy::{asset::AssetMetaCheck, log::Level};
use bevy::log::LogPlugin;
use bevy::prelude::*;

mod beat_timing;
use beat_timing::{BeatConfig, BeatTracker, PressAccept};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }).set(LogPlugin{
            level: Level::TRACE,
            ..default()
        }))
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(Update, process_beat_input)
        .add_observer(combo_handler)
        .add_observer(combo_break_handler)
        .add_observer(combo_text)
        .insert_resource(CurrentCombo::default())
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera3d::default());

    let _level_1 = Song {
        asset_path: "audio/level_2.mp3".to_string(),
        bpm: 115,
        offset: 0.08,
        grace_beats: 8,
    };

    let _level_2 = Song {
        asset_path: "audio/level_1.mp3".to_string(),
        bpm: 124,
        offset: 0.0,
        grace_beats: 8,
    };

    let level_3 = Song {
        asset_path: "audio/level_3.mp3".to_string(),
        bpm: 132,
        offset: 0.04,
        grace_beats: 8,
    };

    let _tracker = BeatTracker::new(BeatConfig {
        bpm: level_3.bpm as f32,
        offset_seconds: level_3.offset,
        leniency_seconds: LENIENCY,
        grace_beats: level_3.grace_beats,
    });

    let control_song = Song {
        asset_path: "audio/control.mp3".to_string(),
        bpm: 180,
        offset: 0.0,
        grace_beats: 24,
    };

    let control = BeatTracker::new(BeatConfig {
        bpm: control_song.bpm as f32,
        offset_seconds: control_song.offset,
        leniency_seconds: LENIENCY,
        grace_beats: control_song.grace_beats,
    });

    commands.spawn((
        AudioPlayer::new(asset_server.load(&control_song.asset_path)),
        control_song,
        BeatTracking { tracker: control },
    ));
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: percent(50),
            height: percent(60),
            ..default()
        },
        BackgroundColor(Color::srgba(0.8, 0.8, 0.8, 0.8)),
        Text::default(),
        HintText,
    ));
}

#[derive(Component)]
struct HintText;

const LENIENCY: f32 = 0.1; // in seconds

#[derive(Clone, Component)]
struct Song {
    asset_path: String,
    bpm: usize,
    offset: f32,        // in seconds
    grace_beats: usize, // number of beats in the start that does not count
}

#[derive(Event)]
enum BeatEvent {
    On(Move),
    Off(Move),
    Missed,
}
enum Beat {
    On,
    Off,
    Missed,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
enum Move {
    Qoogie,
    Woogie,
    Eoogie,
    Roogie,
    Aoogie,
    Soogie,
    Doogie,
    Foogie,
    Shruggie, //Shrug
}

impl From<KeyCode> for Move {
    fn from(key: KeyCode) -> Self {
        match key {
            KeyCode::KeyQ => Move::Qoogie,
            KeyCode::KeyW => Move::Woogie,
            KeyCode::KeyE => Move::Eoogie,
            KeyCode::KeyR => Move::Roogie,
            KeyCode::KeyA => Move::Aoogie,
            KeyCode::KeyS => Move::Soogie,
            KeyCode::KeyD => Move::Doogie,
            KeyCode::KeyF => Move::Foogie,
            _ => Move::Shruggie,
        }
    }
}

impl From<&KeyCode> for Move {
    fn from(key: &KeyCode) -> Self {
        match key {
            KeyCode::KeyQ => Move::Qoogie,
            KeyCode::KeyW => Move::Woogie,
            KeyCode::KeyE => Move::Eoogie,
            KeyCode::KeyR => Move::Roogie,
            KeyCode::KeyA => Move::Aoogie,
            KeyCode::KeyS => Move::Soogie,
            KeyCode::KeyD => Move::Doogie,
            KeyCode::KeyF => Move::Foogie,
            _ => Move::Shruggie,
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Move::Qoogie => "Qoogie",
            Move::Woogie => "Woogie",
            Move::Eoogie => "Eoogie",
            Move::Roogie => "Roogie",
            Move::Aoogie => "Aoogie",
            Move::Soogie => "Soogie",
            Move::Doogie => "Doogie",
            Move::Foogie => "Foogie",
            Move::Shruggie => "Shruggie",
        };
        write!(f, "{}", s)
    }
}

impl Move {
    fn get_combos(&self) -> &'static [Move] {
        use crate::Move::*;
        match self {
            Qoogie => &[Woogie, Eoogie, Foogie, Aoogie],
            Woogie => &[Doogie],
            Eoogie => &[Soogie, Doogie],
            Roogie => &[Aoogie],
            Aoogie => &[Qoogie],
            Soogie => &[Roogie],
            Doogie => &[Eoogie],
            Foogie => &[Woogie],
            Shruggie => &[],
        }
    }
}

#[derive(Event)]
struct ComboBreakEvent(Combo);
type Combo = Vec<Move>;

#[derive(Resource)]
struct CurrentCombo {
    moves: Combo,
}
impl Default for CurrentCombo {
    fn default() -> Self {
        CurrentCombo { moves: Vec::new() }
    }
}

fn combo_break_handler(mut event: On<ComboBreakEvent>) {
    info!("Combo BROKEN! Moves: {:?}", event.event().0);
}

fn combo_handler(event: On<BeatEvent>, mut combo: ResMut<CurrentCombo>, mut commands: Commands) {
    match event.event() {
        BeatEvent::On(mv) => {
            let combo_string = combo
                .moves
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join(" -> ");
            info!("Combo: {}", combo_string);

            if let Some(last_move) = combo.moves.last() {
                if last_move.get_combos().contains(mv) {
                    combo.moves.push(mv.clone()); // Continue combo
                } else {
                    commands.trigger(ComboBreakEvent(combo.moves.clone())); // Trigger combo break event
                    combo.moves.clear(); // Combo break
                }
            } else {
                combo.moves.push(mv.clone()); // Start new combo
            }
        }
        BeatEvent::Off(_) | BeatEvent::Missed => {
            commands.trigger(ComboBreakEvent(combo.moves.clone()));
            combo.moves.clear();
        }
    }
}

fn combo_text(
    event: On<BeatEvent>,
    combo: Res<CurrentCombo>,
    mut query: Query<&mut Text, With<HintText>>,
) {
    if let BeatEvent::On(mv) = event.event() {
        let text = query.single_mut().unwrap().into_inner();
        let combo_string = combo
            .moves
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join(" -> ");
        **text = combo_string;
    }
}

#[derive(Component)]
struct BeatStatistics {
    beats: Vec<Vec<(f32, Beat)>>, // (offset, beat)
}

#[derive(Component)]
struct BeatTracking {
    tracker: BeatTracker,
}

fn process_beat_input(
    mut commands: Commands,
    query: Single<(&AudioSink, &Song, &mut BeatTracking)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (sink, _song, mut tracking) = query.into_inner();

    for event in keys.get_just_pressed() {
        let elapsed_time = sink.position().as_secs_f32();
        let press = tracking.tracker.register_press(elapsed_time);

        // Not on beat, combo break
        if !press.judgment.on_beat {
            commands.trigger(BeatEvent::Off(event.into()));
            trace!(
                "OFF! ({:?}): {} [{}]",
                event, press.judgment.error_seconds, press.judgment.beat_index
            );
            continue;
        }

        // Has already made an accepted move this beat
        if let PressAccept::Duplicate = press.accept {
            trace!(
                "DUPLICATE PRESS ({:?}) beat [{}] (error: {})",
                event,
                press.judgment.beat_index,
                press.judgment.error_seconds
            );
            continue;
        }

        trace!(
            "ON! ({:?}): {} [{}]",
            event, press.judgment.error_seconds, press.judgment.beat_index
        );
        // commands.trigger(BeatEvent::On(event.into()));
    }

    // let elapsed_time = sink.position().as_secs_f32();

    // for event in keys.get_just_pressed() {
    //     let press = tracking.tracker.register_press(elapsed_time);
    //     let beat_index = press.judgment.beat_index;
    //     let on_beat = press.judgment.on_beat;
    //     let offset = press.judgment.error_seconds;

    //     if press.accept == PressAccept::Duplicate {
    //         info!("DUPLICATE PRESS ({:?}) beat [{}]", event, beat_index);
    //         continue;
    //     }

    //     if beat_stats.beats.len() <= beat_index {
    //         beat_stats.beats.resize_with(beat_index + 1, Vec::new);
    //     }

    //     if on_beat {
    //         info!("ON! ({:?}): {} [{}]", event, offset, beat_index);
    //         beat_stats.beats[beat_index].push((offset, Beat::On));
    //         commands.trigger(BeatEvent::On((*event).into()));
    //     } else {
    //         info!("OFF! ({:?}): {} [{}]", event, offset, beat_index);
    //         beat_stats.beats[beat_index].push((offset, Beat::Off));
    //         commands.trigger(BeatEvent::Off((*event).into()));
    //     }
    // }
}

// fn check_for_missed_beat(
//     mut commands: Commands,
//     music_controller: Single<(&AudioSink, &Song, &mut BeatStatistics, &mut BeatTracking)>,
// ) {
//     let (sink, _song, mut stats, mut tracking) = music_controller.into_inner();

//     let elapsed_time = sink.position().as_secs_f32();

//     for miss in tracking.tracker.poll_missed(elapsed_time) {
//         let beat_index = miss.beat_index;
//         let offset = miss.late_by_seconds;
//         trace!("MISSED! {}, index: {}", offset, beat_index);

//         if stats.beats.len() <= beat_index {
//             stats.beats.resize_with(beat_index + 1, Vec::new);
//         }
//         stats.beats[beat_index].push((offset, Beat::Missed));
//         commands.trigger(BeatEvent::Missed);
//     }
// }
