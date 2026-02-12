use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (press_beat, check_for_missed_beat))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera3d::default());
    commands.spawn(Sprite {
        image: asset_server.load("ducky.png"),
        ..Default::default()
    });
    let level_1 = Song {
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

    commands.spawn((
        AudioPlayer::new(asset_server.load(&level_3.asset_path)),
        level_3,
        BeatStatistics { beats: Vec::new() },
    ));
}

const LENIENCY: f32 = 0.1; // in seconds

#[derive(Clone, Component)]
struct Song {
    asset_path: String,
    bpm: usize,
    offset: f32,        // in seconds
    grace_beats: usize, // number of beats in the start that does not count
}

#[derive(Component, Resource)]
struct Playback {
    song: Song,
    elapsed_time: f32, // in seconds
}

// fn get_keys(keys: Res<ButtonInput<KeyCode>>) {
//     for event in keys.get_just_pressed() {
//         info!("{:?}", event);
//     }
// }
#[derive(Event)]
enum Beat {
    On,
    Off,
    Missed,
}

#[derive(Component)]
enum Move {
    Boogie,
    Woogie,
    Schmoogie,
    Guggie,
    Soogie,
    Wauggie,
    BoogieWoogie,
}

#[derive(Component)]
struct BeatStatistics {
    beats: Vec<Vec<(f32, Beat)>>, // (offset, beat)
}

fn press_beat(
    music_controller: Single<(&AudioSink, &Song, &mut BeatStatistics)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (sink, song, mut beat_stats) = music_controller.into_inner();
    let elapsed_time = sink.position().as_secs_f32();
    let beat_index = get_beat_index(song, elapsed_time);
    let (on_beat, offset) = judge_beat(song, elapsed_time);

    if let None = beat_stats.beats.get_mut(beat_index) {
        beat_stats.beats.push(Vec::new());
    }

    for event in keys.get_just_pressed() {
        if on_beat {
            info!("ON! ({:?}): {} [{}]", event, offset, beat_index);
            beat_stats.beats[beat_index].push((offset, Beat::On));
        } else {
            info!("OFF! ({:?}): {} [{}]", event, offset, beat_index);
            beat_stats.beats[beat_index].push((offset, Beat::Off));
        }
    }
}
///
/// last_state: wheter last check was deemed on of off beat.
fn check_for_missed_beat(
    music_controller: Single<(&AudioSink, &Song, &mut BeatStatistics)>,
    mut last_state: Local<bool>,
) {
    let (sink, song, mut stats) = music_controller.into_inner();

    let elapsed_time = sink.position().as_secs_f32();
    let (on_beat, offset) = judge_beat(song, elapsed_time);
    let beat_index = get_beat_index(song, elapsed_time) - 1;

    // When stats[beat_index] is empty and last_state goes from true to false.
    if !on_beat && *last_state && stats.beats.get(beat_index).map_or(true, |b| b.is_empty()) {
        info!("MISSED! {}, index: {}", offset, beat_index);
        if let Some(beat_vec) = stats.beats.get_mut(beat_index) {
            beat_vec.push((offset, Beat::Missed));
        } else {
            stats.beats.push(vec![(offset, Beat::Missed)]);
        }
    }
    *last_state = on_beat;
}

/// Function to get a timestamp to determine if it is on beat or not.
/// Returns a tuple of (on_beat, error) where on_beat is a boolean indicating if the input is on beat and error is the time difference from the nearest beat in seconds.
/// Error is positive if the input is after the beat and negative if it is before the beat.
fn judge_beat(song: &Song, elapsed_time: f32) -> (bool, f32) {
    let beat_period = 60.0 / song.bpm as f32;

    let phase = (elapsed_time + song.offset + (beat_period / 2.0)) % beat_period;
    //phase is the distance from the last beat, and beat_period - phase is the distance to the next beat. We want the smaller of the two.
    //Added + beat_period / 2.0 because otherwise it was the backbeat for some reason
    let mut error = phase.min(beat_period - phase);
    let on_beat = error <= LENIENCY;

    if phase > beat_period / 2.0 {
        error = -error;
    }

    (on_beat, error)
}

fn get_beat_index(song: &Song, elapsed_time: f32) -> usize {
    let beat_period = 60.0 / song.bpm as f32;
    ((elapsed_time + song.offset) / beat_period).round() as usize
}
