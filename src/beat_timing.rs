use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub struct BeatConfig {
    pub bpm: f32,
    pub offset_seconds: f32,
    pub leniency_seconds: f32,
    pub grace_beats: usize,
}

impl BeatConfig {
    pub fn beat_period_seconds(&self) -> f32 {
        60.0 / self.bpm
    }

    /// Time (in seconds) when beat `beat_index` occurs.
    pub fn beat_time_seconds(&self, beat_index: usize) -> f32 {
        (beat_index as f32) * self.beat_period_seconds() - self.offset_seconds
    }

    /// Judges the closest beat to `elapsed_seconds`.
    ///
    /// `error_seconds` is signed:
    /// - positive if the input is after the beat
    /// - negative if the input is before the beat
    pub fn judge(&self, elapsed_seconds: f32) -> Judgment {
        let beat_period = self.beat_period_seconds();

        // This matches your previous logic: shift by half a beat so that modulo math
        // picks the nearest beat rather than the previous beat.
        let phase = (elapsed_seconds + self.offset_seconds + (beat_period / 2.0)) % beat_period;

        let mut error = phase.min(beat_period - phase);
        let on_beat = error <= self.leniency_seconds;

        if phase > beat_period / 2.0 {
            error = -error;
        }

        // Use the same indexing semantics you already had.
        let beat_index = ((elapsed_seconds + self.offset_seconds) / beat_period).round() as usize;

        Judgment {
            beat_index,
            on_beat,
            error_seconds: error,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Judgment {
    pub beat_index: usize,
    pub on_beat: bool,
    pub error_seconds: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressAccept {
    Accepted,
    Duplicate,
}

#[derive(Debug, Clone, Copy)]
pub struct PressResult {
    pub judgment: Judgment,
    pub accept: PressAccept,
    /// Whether this beat should count for gameplay/scoring (i.e., not in grace beats).
    pub counts: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct MissedBeat {
    pub beat_index: usize,
    /// How late we are (seconds) past the beat time.
    pub late_by_seconds: f32,
}

/// Tracks beat resolution over time so you can:
/// - judge a press as on/off-beat
/// - prevent multiple presses per beat
/// - detect missed beats as time advances
#[derive(Debug, Clone)]
pub struct BeatTracker {
    config: BeatConfig,
    // A beat index is "resolved" once we accept the first press for that beat,
    // or once we emit a missed event for it.
    //
    // Important: we only resolve a beat on an *on-beat* press.
    // An off-beat press should not prevent a later on-beat press from counting.
    resolved_beats: HashSet<usize>,
    // Next beat index we should consider for missed-beat detection.
    next_miss_check: usize,
}

impl BeatTracker {
    pub fn new(config: BeatConfig) -> Self {
        Self {
            config,
            resolved_beats: HashSet::new(),
            next_miss_check: config.grace_beats,
        }
    }

    pub fn config(&self) -> BeatConfig {
        self.config
    }

    pub fn reset(&mut self, config: BeatConfig) {
        self.config = config;
        self.resolved_beats.clear();
        self.next_miss_check = config.grace_beats;
    }

    /// Register a button press at `elapsed_seconds`.
    ///
    /// Enforces "no double press on a beat" by accepting only the first press
    /// for a given beat index.
    pub fn register_press(&mut self, elapsed_seconds: f32) -> PressResult {
        let judgment = self.config.judge(elapsed_seconds);
        let counts = judgment.beat_index >= self.config.grace_beats;

        // Only "consume" the beat if the press is actually on-beat.
        // Otherwise, allow a later on-beat press for that same beat index.
        let accept = if judgment.on_beat {
            if self.resolved_beats.contains(&judgment.beat_index) {
                PressAccept::Duplicate
            } else {
                self.resolved_beats.insert(judgment.beat_index);
                PressAccept::Accepted
            }
        } else {
            PressAccept::Accepted
        };

        PressResult {
            judgment,
            accept,
            counts,
        }
    }

    /// Poll for beats that are now impossible to hit (i.e., their leniency window has passed)
    /// and were never pressed.
    pub fn poll_missed(&mut self, elapsed_seconds: f32) -> Vec<MissedBeat> {
        let mut missed = Vec::new();
        let beat_period = self.config.beat_period_seconds();
        let leniency = self.config.leniency_seconds;

        loop {
            let beat_index = self.next_miss_check;
            let beat_time = self.config.beat_time_seconds(beat_index);

            // Once we are past (beat_time + leniency), that beat can no longer be hit.
            if elapsed_seconds <= beat_time + leniency {
                break;
            }

            if !self.resolved_beats.contains(&beat_index) {
                self.resolved_beats.insert(beat_index);
                missed.push(MissedBeat {
                    beat_index,
                    late_by_seconds: elapsed_seconds - beat_time,
                });
            }

            self.next_miss_check = self
                .next_miss_check
                .saturating_add(1)
                // protect against weird configs (e.g. bpm=0) causing infinite loops
                .max(self.config.grace_beats);

            // extra safety: if beat_period is pathological, bail
            if !beat_period.is_finite() || beat_period <= 0.0 {
                break;
            }
        }

        missed
    }
}
