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