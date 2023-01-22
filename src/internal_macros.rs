/** This file is for internal */

macro_rules! forward_ref_op {
    (- $t:ty) => {
        forward_ref_op!(impl Neg, neg for $t);
    };
    ($t:ty, += $u:ty) => {
        forward_ref_op!(impl AddAssign, add_assign for $t, $u);
    };
    ($t:ty, -= $u:ty) => {
        forward_ref_op!(impl SubAssign, sub_assign for $t, $u);
    };
    ($t:ty, &= $u:ty) => {
        forward_ref_op!(impl BitAndAssign, bitand_assign for $t, $u);
    };
    ($t:ty, |= $u:ty) => {
        forward_ref_op!(impl BitOrAssign, bitor_assign for $t, $u);
    };
    ($t:ty, ^= $u:ty) => {
        forward_ref_op!(impl BitXorAssign, bitxor_assign for $t, $u);
    };
    ($t:ty, + $u:ty, = $o:ty) => {
        forward_ref_op!(impl Add, add for $t, $u, = $o);
    };
    ($t:ty, - $u:ty, = $o:ty) => {
        forward_ref_op!(impl Sub, sub for $t, $u, = $o);
    };
    ($t:ty, & $u:ty, = $o:ty) => {
        forward_ref_op!(impl BitAnd, bitand for $t, $u, = $o);
    };
    ($t:ty, | $u:ty, = $o:ty) => {
        forward_ref_op!(impl BitOr, bitor for $t, $u, = $o);
    };
    ($t:ty, ^ $u:ty, = $o:ty) => {
        forward_ref_op!(impl BitXor, bitxor for $t, $u, = $o);
    };
    // for unary operator
    (impl $op:ident, $method:ident for $t:ty) => {
        impl std::ops::$op for &$t {
            type Output = $t;

            #[inline]
            fn $method(self) -> $t {
                <$t>::$method(*self)
            }
        }
    };
    // for assign operator
    (impl $op:ident, $method:ident for $t:ty, $u:ty) => {
        impl std::ops::$op<&$u> for $t {
            #[inline]
            fn $method(&mut self, rhs: &$u) {
                <$t>::$method(self, *rhs)
            }
        }
    };
    // for binary operator
    (impl $op:ident, $method:ident for $t:ty, $u:ty, = $o:ty) => {
        impl std::ops::$op<&$u> for $t {
            type Output = $o;

            #[inline]
            fn $method(self, rhs: &$u) -> $o {
                <$t>::$method(self, *rhs)
            }
        }

        impl std::ops::$op<$u> for &$t {
            type Output = $o;

            #[inline]
            fn $method(self, rhs: $u) -> $o {
                <$t>::$method(*self, rhs)
            }
        }

        impl std::ops::$op<&$u> for &$t {
            type Output = $o;

            #[inline]
            fn $method(self, rhs: &$u) -> $o {
                <$t>::$method(*self, *rhs)
            }
        }
    };
}

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

pub(crate) use forward_ref_op;
pub(crate) use forward_impl_try_from;
pub(crate) use forward_impl_from;
