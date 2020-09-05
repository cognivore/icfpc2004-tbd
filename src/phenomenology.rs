// TODO: remove when it's implemented
#![allow(unused_imports, unused_variables, unused_mut)]

use crate::geography::Markers;

#[derive(Debug)]
pub enum SenseCondition {
    Friend,
    Foe,
    FriendWithFood,
    FoeWithFood,
    Food,
    Rock,
    Marker(Marker),
    FoeMarker,
    Home,
    FoeHome,
}

#[derive(Copy, Clone, Debug)]
pub struct Marker(pub u8);
