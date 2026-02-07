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
        .add_systems(Update, (get_keys, beat))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera3d::default());
    commands.spawn(Sprite {
        image: asset_server.load("ducky.png"),
        ..Default::default()
    });

    let song = Song {
        asset_path: "audio/level_3.mp3".to_string(),
        bpm: 132,
        offset: 0.13,
    };

    commands.spawn((AudioPlayer::new(asset_server.load(&song.asset_path)), song));
}

const LENIENCY: f32 = 0.1; // in seconds

#[derive(Clone, Component)]
struct Song {
    asset_path: String,
    bpm: usize,
    offset: f32, // in seconds
}

#[derive(Component, Resource)]
struct Playback {
    song: Song,
    elapsed_time: f32, // in seconds
}

fn get_keys(keys: Res<ButtonInput<KeyCode>>) {
    for event in keys.get_just_pressed() {
        info!("{:?}", event);
    }
}

fn beat(music_controller: Single<(&AudioSink, &Song)>, keys: Res<ButtonInput<KeyCode>>) {
    let (sink, song) = music_controller.into_inner();

    let elapsed_time = sink.position().as_secs_f32();
    let (on_beat, offset) = judge_beat(song, elapsed_time);
    for event in keys.get_just_pressed() {
        if on_beat {
            info!("ON! ({:?}): {}", event, offset);
        } else {
            info!("OFF! ({:?}): {}", event, offset);
        }
    }
}

fn judge_beat(song: &Song, elapsed_time: f32) -> (bool, f32) {
    let beat_period = 60.0 / song.bpm as f32;

    if ((elapsed_time % beat_period) + song.offset - beat_period / 2.0).abs() < LENIENCY {
        // info!(
        //     "ON beat: {}",
        //     (elapsed_time % beat_period) + song.offset - beat_period / 2.0
        // );
        (
            true,
            (elapsed_time % beat_period) + song.offset - beat_period / 2.0,
        )
    } else {
        // info!(
        //     "OFF beat: {}",
        //     (elapsed_time % beat_period) + song.offset - beat_period / 2.0
        // );
        (
            false,
            (elapsed_time % beat_period) + song.offset - beat_period / 2.0,
        )
    }
}
