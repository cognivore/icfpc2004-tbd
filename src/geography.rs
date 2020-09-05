use bitmaps::Bitmap;
use typenum::U6;

use std::collections::HashMap;

use crate::biology::{
    Color,
    Ant,
};

#[derive(Clone)]
pub enum MapToken {
    Rock,
    Clear(Contents),
}

#[derive(Clone)]
pub struct Contents {
    pub ant : Option<Ant>,
    pub food : Food,
    pub markers : HashMap<Color, Markers>,
}

#[derive(Clone)]
pub struct Markers(pub Bitmap<U6>);

#[derive(Clone)]
pub struct Food(pub u8);
