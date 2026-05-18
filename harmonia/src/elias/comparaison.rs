use crate::notes::{Note, GUITAR};

// Tolérance d'accordage en cents.
// 5 cents = accordé assez précisément pour un accordeur simple.
pub const IN_TUNE_TOLERANCE_CENTS: f32 = 5.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TuningStatus {
    TooLow,
    InTune,
    TooHigh,
}

#[derive(Clone, Copy, Debug)]
pub struct TuningResult {
    pub note: &'static Note,
    pub detected_frequency: f32,
    pub target_frequency: f32,
    pub cents: f32,
    pub status: TuningStatus,
}

pub fn closest_note(freq: f32) -> &'static Note {
    let mut closest = &GUITAR[0];
    let mut min_cents = cents_deviation(freq, closest.frequency).abs();

    for note in GUITAR.iter() {
        let cents = cents_deviation(freq, note.frequency).abs();
        if cents < min_cents {
            min_cents = cents;
            closest = note;
        }
    }

    closest
}

pub fn analyze_frequency(freq: f32) -> Option<TuningResult> {
    if !freq.is_finite() || freq <= 0.0 {
        return None;
    }

    let note = closest_note(freq);
    let cents = cents_deviation(freq, note.frequency);

    let status = if cents.abs() <= IN_TUNE_TOLERANCE_CENTS {
        TuningStatus::InTune
    } else if cents < 0.0 {
        TuningStatus::TooLow
    } else {
        TuningStatus::TooHigh
    };

    Some(TuningResult {
        note,
        detected_frequency: freq,
        target_frequency: note.frequency,
        cents,
        status,
    })
}

pub fn compare_frequency(freq: f32) -> String {
    match analyze_frequency(freq) {
        Some(result) => match result.status {
            TuningStatus::InTune => format!(
                "{} accordée ({:.2} Hz, écart {:+.1} cents)",
                result.note.name,
                result.detected_frequency,
                result.cents
            ),
            TuningStatus::TooLow => format!(
                "{} trop grave ({:.2} Hz, cible {:.2} Hz, écart {:+.1} cents)",
                result.note.name,
                result.detected_frequency,
                result.target_frequency,
                result.cents
            ),
            TuningStatus::TooHigh => format!(
                "{} trop aiguë ({:.2} Hz, cible {:.2} Hz, écart {:+.1} cents)",
                result.note.name,
                result.detected_frequency,
                result.target_frequency,
                result.cents
            ),
        },
        None => "Aucune note détectée".to_string(),
    }
}

// Écart en cents entre la fréquence détectée et la fréquence cible.
// Négatif = trop grave, positif = trop aigu.
pub fn cents_deviation(detected: f32, target: f32) -> f32 {
    if detected <= 0.0 || target <= 0.0 {
        return 0.0;
    }

    1200.0 * (detected / target).log2()
}
