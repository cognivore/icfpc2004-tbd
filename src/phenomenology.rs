#[derive(Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Marker(u8);

impl Marker {
    pub fn new(i: u8) -> Self {
        assert!(i < 6);
        Marker(i)
    }
}