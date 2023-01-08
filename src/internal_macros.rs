/** This file is for internal */

macro_rules! forward_impl_try_from {
    ($t:ty, $e:ty, from $u:ty) => {
        impl TryFrom<$u> for $t {
            type Error = $e;

            #[inline]
            fn try_from(value: $u) -> Result<Self, Self::Error> {
                Self::try_from(&value)
            }
        }
    };
}

macro_rules! forward_impl_from {
    ($t:ty, from $u:ty) => {
        impl From<$u> for $t {
            #[inline]
            fn from(value: $u) -> Self {
                Self::from(&value)
            }
        }
    };
}

pub(crate) use forward_impl_try_from;
pub(crate) use forward_impl_from;
