use nom::{number::complete::be_u16, IResult};
use smallvec::SmallVec;

use crate::{constant_pool::ConstantPoolIndexRaw, parser::ParseData};

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
    };
}

pub(crate) fn constant_pool_index_raw<T>(
    i: ParseData,
) -> IResult<ParseData, ConstantPoolIndexRaw<T>> {
    let (i, v) = be_u16(i)?;
    Ok((i, ConstantPoolIndexRaw::new(v)))
}

/// Count but for small vec
pub(crate) fn count_sv<const N: usize, I, O, E, F>(
    mut f: F,
    count: usize,
) -> impl FnMut(I) -> IResult<I, SmallVec<[O; N]>, E>
where
    I: Clone + PartialEq,
    F: nom::Parser<I, O, E>,
    E: nom::error::ParseError<I>,
{
    move |i: I| {
        let mut input = i.clone();
        let mut res = SmallVec::with_capacity(count);

        for _ in 0..count {
            let input_ = input.clone();
            match f.parse(input_) {
                Ok((i, o)) => {
                    res.push(o);
                    input = i;
                }
                Err(nom::Err::Error(e)) => {
                    return Err(nom::Err::Error(E::append(
                        i,
                        nom::error::ErrorKind::Count,
                        e,
                    )));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok((input, res))
    }
}
