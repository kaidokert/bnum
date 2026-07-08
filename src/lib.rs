#![allow(incomplete_features)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(
    all(test, nightly),
    feature(
        widening_mul,
        signed_bigint_helpers,
        int_roundings,
        uint_bit_width,
        wrapping_next_power_of_two,
        f16,
        f128,
        int_from_ascii
    )
)]
#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(feature = "arbitrary", feature = "quickcheck")), no_std)]
// TODO: MAKE SURE NO_STD IS ENABLED WHEN PUBLISHING NEW VERSION

// TODO: create issue on gh about v1.0 release. problem is that crates like rand aren't in 1.x yet

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;

mod integer;

pub mod cast;
mod doc;
pub mod errors;
mod helpers;
#[doc(hidden)]
pub mod literal_parse;
pub mod prelude;
mod digits;
mod overflow;

// #[cfg(feature = "float")]
// mod float;

#[cfg(feature = "rand")]
pub mod random;

pub mod types;

#[cfg(test)]
mod test;

type Exponent = u32;
type Byte = u8;

pub use integer::{Int, Integer, Uint};
pub use overflow::OverflowMode;
pub use types::{WI128, WI1024, WI2048, WI256, WI4096, WI512, WI8192, WU128, WU1024, WU2048, WU256, WU4096, WU512, WU8192};
#[cfg(any(feature = "numtraits", feature = "num-traits-only"))]
pub use integer::ByteArray;

// #[cfg(feature = "float")]
// pub use float::Float;