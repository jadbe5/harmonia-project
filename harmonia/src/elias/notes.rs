pub struct Note
{
    pub name: &'static str,
    pub frequency: f32,
}

pub const GUITAR: [Note; 6] =
    [ 
        Note { name: "E2", frequency: 82.41 },
        Note { name: "A2", frequency: 110.00 },
        Note { name: "D3", frequency: 146.83 },
        Note { name: "G3", frequency: 196.00 },
        Note { name: "B3", frequency: 246.94 },
        Note { name: "E4", frequency: 329.63 }
    ];


