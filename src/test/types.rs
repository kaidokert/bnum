pub mod big_types {
	macro_rules! big_types_modules {
		($bits: literal) => {
			pub mod u8 {
				pub type UTEST = crate::BUintU8D<{$bits / 8}>;
				pub type ITEST = crate::BIntU8D<{$bits / 8}>;
			}
			pub mod u16 {
				pub type UTEST = crate::BUintU16D<{$bits / 16}>;
				pub type ITEST = crate::BIntU16D<{$bits / 16}>;
			}
			pub mod u32 {
				pub type UTEST = crate::BUintU32D<{$bits / 32}>;
				pub type ITEST = crate::BIntU32D<{$bits / 32}>;
			}
			pub mod u64 {
				pub type UTEST = crate::BUint<{$bits / 64}>;
				pub type ITEST = crate::BInt<{$bits / 64}>;
			}
		}
	}

	#[cfg(test_int_bits = "64")]
	big_types_modules!(64);

	#[cfg(not(test_int_bits = "64"))]
	big_types_modules!(128);
}

#[cfg(test_int_bits = "64")]
mod small_types {
	#[allow(non_camel_case_types)]
	pub type utest = u64;

	#[allow(non_camel_case_types)]
	pub type itest = i64;	
}

#[cfg(not(test_int_bits = "64"))]
mod small_types {
	#[allow(non_camel_case_types)]
	pub type utest = u128;

	#[allow(non_camel_case_types)]
	pub type itest = i128;	
}

pub use small_types::*;
pub use core::primitive::*;