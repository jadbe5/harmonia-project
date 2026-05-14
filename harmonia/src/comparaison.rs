use crate::notes::GUITAR;
use crate::notes::Note;

pub fn closest_note(freq: f32) -> &'static Note
{
    let mut closest = &GUITAR[0];
    let mut min_diff = (freq - closest.frequency).abs();

    for note in GUITAR.iter() 
    {
        let diff = (freq - note.frequency).abs();
        if diff < min_diff
        {
            min_diff = diff;
            closest = note;
        }
    }

    closest
}

pub fn compare_frequency(freq: f32) -> String 
{
    let note = closest_note(freq);
    if (freq - note.frequency).abs() < 0.5 
    {
        format!("{} parfaite !", note.name)
    }
    else if freq < note.frequency 
    {
        format!("{} est trop grave", note.name)
    } 
    else 
    {
        format!("{} est trop aigue", note.name)
    }
}

/// Retourne l'écart en cents entre la fréquence détectée et la fréquence cible.
/// Positif = trop aigu, négatif = trop grave. Plage typique : -50..+50
pub fn cents_deviation(detected: f32, target: f32) -> f32
{
    if detected <= 0.0 || target <= 0.0 {
        return 0.0;
    }
    1200.0 * (detected / target).log2()
}
