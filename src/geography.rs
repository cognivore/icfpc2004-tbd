use bitmaps::Bitmap;
use typenum::U6;

use std::collections::HashMap;

use crate::biology::{
    Color,
    Ant,
};

#[derive(Debug, Clone)]
pub enum MapToken {
    Rock,
    Clear(Contents),
}

#[derive(Debug, Clone)]
pub struct Contents {
    pub ant : Option<Ant>,
    pub anthill : Option<Color>,
    pub food : Food,
    pub markers : Markers,
}

impl Contents {
    pub fn empty() -> Contents {
        Contents{
            ant: None,
            anthill: None,
            food: Food(0),
            markers: Markers::empty(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Markers(pub HashMap<Color, Bitmap<U6>>);

impl Markers {
    pub fn empty() -> Markers {
        let mut h : HashMap<Color, Bitmap<U6>> = HashMap::new();
        h.insert(Color::Red, Bitmap::new());
        h.insert(Color::Black, Bitmap::new());
        Markers(h)
    }
}

#[derive(Clone, Debug)]
pub struct Food(pub u16);
