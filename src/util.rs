#[macro_export]
macro_rules! impl_from {
    (enum $t:ident => $v:ident :: $n:ident) => {
        impl From<$t> for $v {
            fn from(v: $t) -> $v {
                $v::$n(v)
            }
        }
    };
}

/// Implements from $v for the enum type
/// And implements TryFrom<enum type> for $v
/// The error type is assumed to be just a value to use on failure
#[macro_export]
macro_rules! impl_from_try_reverse {
    (enum $t:ident => $v:ident :: $n:ident ; $err:ident) => {
        impl From<$t> for $v {
            fn from(v: $t) -> $v {
                $v::$n(v)
            }
        }

        impl std::convert::TryFrom<$v> for $t {
            type Error = $err;
            fn try_from(v: $v) -> Result<$t, Self::Error> {
                match v {
                    $v::$n(v) => Ok(v),
                    _ => Err($err),
                }
            }
        }

        impl<'a> std::convert::TryFrom<&'a $v> for &'a $t {
            type Error = $err;
            fn try_from(v: &'a $v) -> Result<&'a $t, Self::Error> {
                match v {
                    $v::$n(v) => Ok(v),
                    _ => Err($err),
                }
            }
        }

        impl<'a> std::convert::TryFrom<&'a mut $v> for &'a mut $t {
            type Error = $err;
            fn try_from(v: &'a mut $v) -> Result<&'a mut $t, Self::Error> {
                match v {
                    $v::$n(v) => Ok(v),
                    _ => Err($err),
                }
            }
        }
    }
}