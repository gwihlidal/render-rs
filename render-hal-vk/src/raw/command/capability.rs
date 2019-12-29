use ash;

/// Queue from families with `Transfer` capability supports only few transfer operations.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Transfer;

/// Queue from families with `Graphics` capability supports drawing operations.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Graphics;

/// Queue from families with `Compute` capability supports compute operations.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Compute;

/// Queue from families with `General` capability supports both compute and graphics operations.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct General;

/// Trait to check what capability supports what operations.
pub trait Supports<C> {
    fn supports(self) -> bool;
}

/// Requirements for either graphics or compute capability.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GraphicsOrCompute {}

impl Supports<Transfer> for Transfer {
    fn supports(self) -> bool {
        true
    }
}
impl Supports<Transfer> for Graphics {
    fn supports(self) -> bool {
        true
    }
}
impl Supports<Transfer> for Compute {
    fn supports(self) -> bool {
        true
    }
}
impl Supports<Transfer> for General {
    fn supports(self) -> bool {
        true
    }
}
impl Supports<Transfer> for Capability {
    fn supports(self) -> bool {
        Capability::supports(self, Capability::Transfer)
    }
}

impl Supports<GraphicsOrCompute> for Graphics {
    fn supports(self) -> bool {
        true
    }
}
impl Supports<GraphicsOrCompute> for Compute {
    fn supports(self) -> bool {
        true
    }
}
impl Supports<GraphicsOrCompute> for General {
    fn supports(self) -> bool {
        true
    }
}
impl Supports<GraphicsOrCompute> for Capability {
    fn supports(self) -> bool {
        Capability::supports(self, Capability::Graphics)
            || Capability::supports(self, Capability::Compute)
    }
}

impl Supports<Graphics> for Graphics {
    fn supports(self) -> bool {
        true
    }
}
impl Supports<Graphics> for General {
    fn supports(self) -> bool {
        true
    }
}
impl Supports<Graphics> for Capability {
    fn supports(self) -> bool {
        Capability::supports(self, Capability::Graphics)
    }
}

impl Supports<Compute> for Compute {
    fn supports(self) -> bool {
        true
    }
}
impl Supports<Compute> for General {
    fn supports(self) -> bool {
        true
    }
}
impl Supports<Compute> for Capability {
    fn supports(self) -> bool {
        Capability::supports(self, Capability::Compute)
    }
}

/// Runtime capability value.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Capability {
    Transfer,
    Graphics,
    Compute,
    General,
}

impl From<ash::vk::QueueFlags> for Capability {
    fn from(flags: ash::vk::QueueFlags) -> Self {
        if flags.contains(ash::vk::QueueFlags::COMPUTE) {
            if flags.contains(ash::vk::QueueFlags::GRAPHICS) {
                Capability::General
            } else {
                Capability::Compute
            }
        } else {
            if flags.contains(ash::vk::QueueFlags::GRAPHICS) {
                Capability::Graphics
            } else {
                assert!(flags.contains(ash::vk::QueueFlags::TRANSFER));
                Capability::Transfer
            }
        }
    }
}

impl Capability {
    /// Check if capability supports another one.
    pub fn supports(self, other: Capability) -> bool {
        match (self, other) {
            (_, Capability::Transfer) | (Capability::General, _) => true,
            (left, right) => left == right,
        }
    }

    /// Transform into `Transfer`.
    /// Panics if not supported.
    pub fn transfer(self) -> Transfer {
        assert!(self.supports(Transfer.into()));
        Transfer
    }

    /// Transform into `Graphics`.
    /// Panics if not supported.
    pub fn graphics(self) -> Graphics {
        assert!(self.supports(Graphics.into()));
        Graphics
    }

    /// Transform into `Compute`.
    /// Panics if not supported.
    pub fn compute(self) -> Compute {
        assert!(self.supports(Compute.into()));
        Compute
    }

    /// Transform into `General`.
    /// Panics if not supported.
    pub fn general(self) -> General {
        assert!(self.supports(General.into()));
        General
    }
}

impl From<Transfer> for Capability {
    fn from(_: Transfer) -> Self {
        Capability::Transfer
    }
}
impl From<Graphics> for Capability {
    fn from(_: Graphics) -> Self {
        Capability::Graphics
    }
}
impl From<Compute> for Capability {
    fn from(_: Compute) -> Self {
        Capability::Compute
    }
}
impl From<General> for Capability {
    fn from(_: General) -> Self {
        Capability::General
    }
}
