#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Marker(pub usize);

impl Marker {
    pub fn new(i: usize) -> Self {
        assert!(i < 6);
        Marker(i)
    }
}
