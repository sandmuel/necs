use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MiniTypeId(u16);

impl MiniTypeId {
    pub const MAX: Self = Self(u16::MAX);

    #[inline]
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl Display for MiniTypeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for MiniTypeId {
    fn from(value: usize) -> Self {
        Self(
            u16::try_from(value)
                .unwrap_or_else(|_| panic!("cannot register more than {} nodes", MiniTypeId::MAX)),
        )
    }
}
