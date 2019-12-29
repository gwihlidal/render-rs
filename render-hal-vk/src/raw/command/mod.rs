#![allow(dead_code)]

pub mod buffer;
pub mod capability;
pub mod family;
pub mod fence;
pub mod pool;
pub mod queue;

pub use self::buffer::*;
pub use self::capability::*;
pub use self::family::*;
pub use self::fence::*;
pub use self::pool::*;
pub use self::queue::*;

#[derive(Copy, Clone, Debug)]
pub struct FamilyId<C = Capability> {
    pub(crate) index: u32,
    pub(crate) capability: C,
}

impl FamilyId<Capability> {
    /// Fetch capability of the family.
    pub fn capability(&self) -> Capability {
        self.capability
    }
}

impl<C> PartialEq for FamilyId<C> {
    fn eq(&self, rhs: &Self) -> bool {
        self.index == rhs.index
    }
}

impl<C> Eq for FamilyId<C> {}

impl FamilyId<Capability> {
    pub fn transfer(self) -> FamilyId<Transfer> {
        FamilyId {
            index: self.index,
            capability: self.capability.transfer(),
        }
    }

    pub fn graphics(self) -> FamilyId<Graphics> {
        FamilyId {
            index: self.index,
            capability: self.capability.graphics(),
        }
    }

    pub fn compute(self) -> FamilyId<Compute> {
        FamilyId {
            index: self.index,
            capability: self.capability.compute(),
        }
    }

    pub fn general(self) -> FamilyId<General> {
        FamilyId {
            index: self.index,
            capability: self.capability.general(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct QueueId<C = Capability> {
    pub(crate) index: u32,
    pub(crate) family: FamilyId<C>,
}

impl QueueId<Capability> {
    /// Fetch capability of the queue.
    pub fn capability(&self) -> Capability {
        self.family.capability()
    }
}

impl<C> PartialEq for QueueId<C> {
    fn eq(&self, rhs: &Self) -> bool {
        self.index == rhs.index && self.family == rhs.family
    }
}

impl<C> Eq for QueueId<C> {}

impl QueueId<Capability> {
    pub fn transfer(self) -> QueueId<Transfer> {
        QueueId {
            index: self.index,
            family: self.family.transfer(),
        }
    }

    pub fn graphics(self) -> QueueId<Graphics> {
        QueueId {
            index: self.index,
            family: self.family.graphics(),
        }
    }

    pub fn compute(self) -> QueueId<Compute> {
        QueueId {
            index: self.index,
            family: self.family.compute(),
        }
    }

    pub fn general(self) -> QueueId<General> {
        QueueId {
            index: self.index,
            family: self.family.general(),
        }
    }
}
