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
pub struct Marker(pub usize);

impl Marker {
    pub fn new(i: usize) -> Self {
        assert!(i < 6);
        Marker(i)
    }
}
