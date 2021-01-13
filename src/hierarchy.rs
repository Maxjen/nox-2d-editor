use legion::*;
use smallvec::SmallVec;

pub struct Parent(pub Entity);

pub struct Children(pub SmallVec<[Entity; 8]>);