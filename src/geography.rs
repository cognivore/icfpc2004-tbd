use bitmaps::Bitmap;
use typenum::U6;

use std::collections::HashMap;

use crate::biology::{
    Color,
    Ant,
};

pub enum MapToken {
    Rock,
    Clear(Contents),
}

pub struct Contents {
    pub ant : Option<Ant>,
    pub food : Food,
    pub markers : HashMap<Color, Markers>,
}

pub struct Markers(pub Bitmap<U6>);

pub struct Food(pub u8);
