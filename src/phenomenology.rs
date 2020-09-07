use std::fmt;

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


impl fmt::Display for SenseCondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let SenseCondition::Marker(m) = self {
            write!(f, "Marker {}", m.0)
        }
        else {
            write!(f, "{:?}", self)
        }
    }
}


impl Marker {
    pub fn new(i: usize) -> Self {
        assert!(i < 6);
        Marker(i)
    }
}


impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


