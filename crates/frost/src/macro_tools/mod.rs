macro_rules! concat_reverse {
    ([] $($reversed:tt)*) => { 
        concat!($(stringify!($reversed)),*)
    };
    ([$first:tt $($rest:tt)*] $($reversed:tt)*) => { 
        concat_reverse!([$($rest)*] $first $($reversed)*)
    };
}