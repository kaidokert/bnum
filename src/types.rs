//! Type aliases for unsigned and signed integers.
//! 
//! Each type alias has the same overflow behaviour as that of the primitive integer types, i.e. wrap on overflow if `overflow-checks` are disabled and panic on overflow if `overflow-checks` are enabled.

macro_rules! int_type_doc {
    ($bits: literal, $sign: literal, $aliased: literal) => {
        concat!(
            $bits, "-bit ", $sign, " integer type.",
            "\n\n",
            "Overflow behaviour is the same as that of the primitive integer types, i.e. wrap on overflow if `overflow-checks` are disabled and panic on overflow if `overflow-checks` are enabled.",
            "\n\n",
            "This type is an alias of [`", $aliased, "`](crate::", $aliased, "). See the documentation of [`", $aliased, "`](crate::", $aliased, ") for available methods and behaviour details."
        )
    };
}

macro_rules! int_types {
    { $($bits: literal $u: ident $i: ident; ) *}  => {
        $(
            #[doc = int_type_doc!($bits, "unsigned", "Uint")]
            pub type $u = crate::Uint::<{ crate::literal_parse::get_size_params_from_bits($bits).0 }, { crate::literal_parse::get_size_params_from_bits($bits).1 }>;

            #[doc = int_type_doc!($bits, "signed", "Int")]
            pub type $i = crate::Int::<{ crate::literal_parse::get_size_params_from_bits($bits).0 }, { crate::literal_parse::get_size_params_from_bits($bits).1 }>;
        )*
    };
}

macro_rules! wrapping_int_types {
    { $($bits: literal $wu: ident $wi: ident; ) *}  => {
        $(
            #[doc = concat!(
                $bits, "-bit unsigned integer type that always wraps on overflow, regardless of build profile.",
                "\n\nIntended for crypto field arithmetic where 2's-complement wrapping semantics are required."
            )]
            pub type $wu = crate::Uint::<
                { crate::literal_parse::get_size_params_from_bits($bits).0 },
                { crate::literal_parse::get_size_params_from_bits($bits).1 },
                // OverflowMode::Wrap = 0
                0,
            >;

            #[doc = concat!(
                $bits, "-bit signed integer type that always wraps on overflow, regardless of build profile.",
            )]
            pub type $wi = crate::Int::<
                { crate::literal_parse::get_size_params_from_bits($bits).0 },
                { crate::literal_parse::get_size_params_from_bits($bits).1 },
                0,
            >;
        )*
    };
}

macro_rules! call_types_macro {
    ($name: ident) => {
        $name! {
            128 U128 I128;
            256 U256 I256;
            512 U512 I512;
            1024 U1024 I1024;
            2048 U2048 I2048;
            4096 U4096 I4096;
            8192 U8192 I8192;
        }
    };
}

call_types_macro!(int_types);

wrapping_int_types! {
    128 WU128 WI128;
    256 WU256 WI256;
    512 WU512 WI512;
    1024 WU1024 WI1024;
    2048 WU2048 WI2048;
    4096 WU4096 WI4096;
    8192 WU8192 WI8192;
}

// #[cfg(feature = "float")]
// /// 16-bit floating point type with 10 bits of precision, stored as the binary16 (half precision) format defined in IEEE 754-2019.
// pub type F16 = crate::Float<2, 10>;

// #[cfg(feature = "float")]
// /// 32-bit floating point type with 23 bits of precision, stored as the binary32 (single precision) format defined in IEEE 754-2019.
// pub type F32 = crate::Float<4, 23>;

// #[cfg(feature = "float")]
// /// 64-bit floating point type with 52 bits of precision, stored as the binary64 (double precision) format defined in IEEE 754-2019.
// pub type F64 = crate::Float<8, 52>;

// #[cfg(feature = "float")]
// /// 80-bit floating point type with 64 bits of precision.
// pub type F80 = crate::Float<10, 64>;

// #[cfg(feature = "float")]
// /// 128-bit floating point type with 112 bits of precision, stored as the binary128 (quadruple precision) format defined in IEEE 754-2019.
// pub type F128 = crate::Float<16, 112>;

// #[cfg(feature = "float")]
// /// 256-bit floating point type with 236 bits of precision, stored as the binary256 (octuple precision) format defined in IEEE 754-2019.
// pub type F256 = crate::Float<32, 236>;


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_int_bits {
        { $($bits: literal $u: ident $i: ident; ) *} => {
            $(
                assert_eq!($u::BITS, $bits);
                assert_eq!($i::BITS, $bits);
            )*
        }
    }

    #[test]
    fn test_int_bits() {
        call_types_macro!(assert_int_bits);
    }
}
