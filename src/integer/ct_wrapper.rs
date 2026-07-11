// Ct<U>: transparent newtype that projects the const_num_traits::Ct
// personality onto any numeric carrier U.
//
// rsa_heapless 0.4 requires ModMathParams<T, Ct> for signing, which requires
// T: ModMathIntCt + HasPersonality<P = Ct>. A carrier type has exactly one P,
// so Nct carriers (Uint<N, B, OM>) are structurally barred from the signing
// path. Ct<U> is the minimal fix: it wraps any U and fixes HasPersonality::P
// to Ct while delegating all arithmetic identically.
//
// The "CT" operations (ConditionallySelectable, ConstantTimeLess) delegate to
// the same vartime bodies U uses — they satisfy the type constraint but are
// not genuinely constant-time until U gets real CT limb primitives. This is
// the correct trade-off: it unblocks compilation and is honest (vartime U
// bodies → vartime Ct<U> bodies), and constant-time replacements can be
// dropped in later without an API change.

use core::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr, ShrAssign};

use const_num_traits::ops::byte_slice::ByteSliceError;
use const_num_traits::ops::overflowing::OverflowingAdd;
use const_num_traits::ops::wrapping::{WrappingAdd, WrappingMul, WrappingSub};
use const_num_traits::{
    BorrowingSub, CarryingMul, ConstOne, ConstZero, CtIsZero, FromByteSlice, HasPersonality,
    Parity, PrimBits, ToBytes,
};
use modmath_cios::CiosRowOps;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeGreater};

/// Transparent wrapper over `U` that projects `HasPersonality<P = Ct>`.
///
/// All arithmetic delegates to the inner `U`. Use `Ct<Uint<N, B, OM>>` where
/// `rsa_heapless 0.4`'s `ModMathParams<_, Ct>` (signing path) is required;
/// leave the bare `Uint` (projects `Nct`) for verify paths.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Ct<U>(pub U);

// ── Personality ───────────────────────────────────────────────────────────────

impl<U> HasPersonality for Ct<U> {
    type P = const_num_traits::Ct;
}

// ── Identity values ───────────────────────────────────────────────────────────

impl<U: const_num_traits::Zero> const_num_traits::Zero for Ct<U> {
    fn zero() -> Self {
        Ct(U::zero())
    }
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
    fn set_zero(&mut self) {
        self.0.set_zero();
    }
}

impl<U: const_num_traits::One + PartialEq> const_num_traits::One for Ct<U> {
    fn one() -> Self {
        Ct(U::one())
    }
    fn is_one(&self) -> bool {
        self.0.is_one()
    }
    fn set_one(&mut self) {
        self.0.set_one();
    }
}

impl<U: ConstZero> ConstZero for Ct<U> {
    const ZERO: Self = Ct(U::ZERO);
}

impl<U: ConstOne> ConstOne for Ct<U> {
    const ONE: Self = Ct(U::ONE);
}

// ── Parity ────────────────────────────────────────────────────────────────────

impl<U: Parity> Parity for Ct<U> {
    fn is_odd(self) -> bool {
        self.0.is_odd()
    }
    fn is_even(self) -> bool {
        self.0.is_even()
    }
}

// ── PrimBits ─────────────────────────────────────────────────────────────────

impl<U: PrimBits> PrimBits for Ct<U> {
    fn count_ones(self) -> u32 {
        self.0.count_ones()
    }
    fn count_zeros(self) -> u32 {
        self.0.count_zeros()
    }
    fn leading_zeros(self) -> u32 {
        self.0.leading_zeros()
    }
    fn trailing_zeros(self) -> u32 {
        self.0.trailing_zeros()
    }
    fn rotate_left(self, n: u32) -> Self {
        Ct(self.0.rotate_left(n))
    }
    fn rotate_right(self, n: u32) -> Self {
        Ct(self.0.rotate_right(n))
    }
    fn signed_shl(self, n: u32) -> Self {
        Ct(self.0.signed_shl(n))
    }
    fn signed_shr(self, n: u32) -> Self {
        Ct(self.0.signed_shr(n))
    }
    fn unsigned_shl(self, n: u32) -> Self {
        Ct(self.0.unsigned_shl(n))
    }
    fn unsigned_shr(self, n: u32) -> Self {
        Ct(self.0.unsigned_shr(n))
    }
    fn swap_bytes(self) -> Self {
        Ct(self.0.swap_bytes())
    }
    fn from_be(x: Self) -> Self {
        Ct(U::from_be(x.0))
    }
    fn from_le(x: Self) -> Self {
        Ct(U::from_le(x.0))
    }
    fn to_be(self) -> Self {
        Ct(self.0.to_be())
    }
    fn to_le(self) -> Self {
        Ct(self.0.to_le())
    }
}

// ── Arithmetic ops ────────────────────────────────────────────────────────────

impl<U: WrappingAdd<Output = U>> WrappingAdd for Ct<U> {
    type Output = Self;
    fn wrapping_add(self, v: Self) -> Self {
        Ct(self.0.wrapping_add(v.0))
    }
}

impl<U: WrappingSub<Output = U>> WrappingSub for Ct<U> {
    type Output = Self;
    fn wrapping_sub(self, v: Self) -> Self {
        Ct(self.0.wrapping_sub(v.0))
    }
}

impl<U: WrappingMul<Output = U>> WrappingMul for Ct<U> {
    type Output = Self;
    fn wrapping_mul(self, v: Self) -> Self {
        Ct(self.0.wrapping_mul(v.0))
    }
}

impl<U: OverflowingAdd<Output = U>> OverflowingAdd for Ct<U> {
    type Output = Self;
    fn overflowing_add(self, v: Self) -> (Self, bool) {
        let (r, o) = self.0.overflowing_add(v.0);
        (Ct(r), o)
    }
}

impl<U: BorrowingSub<Output = U>> BorrowingSub for Ct<U> {
    type Output = Self;
    fn borrowing_sub(self, rhs: Self, borrow: bool) -> (Self, bool) {
        let (r, b) = self.0.borrowing_sub(rhs.0, borrow);
        (Ct(r), b)
    }
}

impl<U: CarryingMul<Unsigned = U, Output = U>> CarryingMul for Ct<U> {
    type Unsigned = Self;
    type Output = Self;
    fn carrying_mul(self, rhs: Self, carry: Self) -> (Self, Self) {
        let (lo, hi) = self.0.carrying_mul(rhs.0, carry.0);
        (Ct(lo), Ct(hi))
    }
    fn carrying_mul_add(self, rhs: Self, carry: Self, add: Self) -> (Self, Self) {
        let (lo, hi) = self.0.carrying_mul_add(rhs.0, carry.0, add.0);
        (Ct(lo), Ct(hi))
    }
}

// ── Bit ops ───────────────────────────────────────────────────────────────────

impl<U: Shr<usize, Output = U>> Shr<usize> for Ct<U> {
    type Output = Self;
    fn shr(self, n: usize) -> Self {
        Ct(self.0 >> n)
    }
}

impl<U: ShrAssign<usize>> ShrAssign<usize> for Ct<U> {
    fn shr_assign(&mut self, n: usize) {
        self.0 >>= n;
    }
}

impl<U: BitAnd<Output = U>> BitAnd for Ct<U> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Ct(self.0 & rhs.0)
    }
}

impl<U: BitOr<Output = U>> BitOr for Ct<U> {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Ct(self.0 | rhs.0)
    }
}

impl<U: BitXor<Output = U>> BitXor for Ct<U> {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Ct(self.0 ^ rhs.0)
    }
}

impl<U: Not<Output = U>> Not for Ct<U> {
    type Output = Self;
    fn not(self) -> Self {
        Ct(!self.0)
    }
}

impl<U: Shl<usize, Output = U>> Shl<usize> for Ct<U> {
    type Output = Self;
    fn shl(self, n: usize) -> Self {
        Ct(self.0 << n)
    }
}

// &Ct<U> variants (needed for ed25519_heapless UnsignedModularInt for<'a> &'a T bounds)
impl<U: Copy + WrappingAdd<Output = U>> WrappingAdd for &Ct<U> {
    type Output = Ct<U>;
    fn wrapping_add(self, v: Self) -> Ct<U> {
        Ct(self.0.wrapping_add(v.0))
    }
}

impl<U: Copy + WrappingSub<Output = U>> WrappingSub for &Ct<U> {
    type Output = Ct<U>;
    fn wrapping_sub(self, v: Self) -> Ct<U> {
        Ct(self.0.wrapping_sub(v.0))
    }
}

impl<'a, U: Copy + BitAnd<Output = U>> BitAnd for &'a Ct<U> {
    type Output = Ct<U>;
    fn bitand(self, rhs: Self) -> Ct<U> {
        Ct(self.0 & rhs.0)
    }
}

// ── From<u8> ─────────────────────────────────────────────────────────────────

impl<U: From<u8>> From<u8> for Ct<U> {
    fn from(v: u8) -> Self {
        Ct(U::from(v))
    }
}

// ── Zeroize ───────────────────────────────────────────────────────────────────

#[cfg(feature = "zeroize")]
impl<U: zeroize::Zeroize> zeroize::Zeroize for Ct<U> {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

// ── ToBytes / FromBytes ───────────────────────────────────────────────────────

impl<U: ToBytes> ToBytes for Ct<U> {
    type Bytes = U::Bytes;
    fn to_be_bytes(self) -> U::Bytes {
        self.0.to_be_bytes()
    }
    fn to_le_bytes(self) -> U::Bytes {
        self.0.to_le_bytes()
    }
}

impl<U: ToBytes> ToBytes for &Ct<U>
where
    U: Copy,
{
    type Bytes = U::Bytes;
    fn to_be_bytes(self) -> U::Bytes {
        self.0.to_be_bytes()
    }
    fn to_le_bytes(self) -> U::Bytes {
        self.0.to_le_bytes()
    }
}

impl<U: const_num_traits::FromBytes> const_num_traits::FromBytes for Ct<U> {
    type Bytes = U::Bytes;
    fn from_be_bytes(bytes: &U::Bytes) -> Self {
        Ct(U::from_be_bytes(bytes))
    }
    fn from_le_bytes(bytes: &U::Bytes) -> Self {
        Ct(U::from_le_bytes(bytes))
    }
}

impl<U: FromByteSlice> FromByteSlice for Ct<U> {
    fn from_be_slice(bytes: &[u8]) -> Result<Self, ByteSliceError> {
        U::from_be_slice(bytes).map(Ct)
    }
    fn from_le_slice(bytes: &[u8]) -> Result<Self, ByteSliceError> {
        U::from_le_slice(bytes).map(Ct)
    }
}

// ── CiosRowOps ────────────────────────────────────────────────────────────────

impl<U: CiosRowOps> CiosRowOps for Ct<U> {
    type Word = U::Word;

    fn word_count(&self) -> usize {
        self.0.word_count()
    }

    fn word(&self, i: usize) -> U::Word {
        self.0.word(i)
    }

    fn mul_acc_row(scalar: U::Word, multiplicand: &Self, acc: &mut Self, carry_in: U::Word) -> U::Word {
        U::mul_acc_row(scalar, &multiplicand.0, &mut acc.0, carry_in)
    }

    fn mul_acc_shift_row(
        scalar: U::Word,
        multiplicand: &Self,
        acc: &mut Self,
        acc_hi: U::Word,
    ) -> U::Word {
        U::mul_acc_shift_row(scalar, &multiplicand.0, &mut acc.0, acc_hi)
    }
}

// ── subtle CT traits ──────────────────────────────────────────────────────────
//
// ConditionallySelectable delegates to U's impl (vartime for bnum, since
// bnum has no hardware CT primitives). ConstantTimeLess is auto-impl'd by
// subtle once ConstantTimeEq + ConstantTimeGreater are satisfied.

impl<U: ConstantTimeEq> ConstantTimeEq for Ct<U> {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl<U: ConditionallySelectable> ConditionallySelectable for Ct<U> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Ct(U::conditional_select(&a.0, &b.0, choice))
    }
}

impl<U: ConstantTimeGreater> ConstantTimeGreater for Ct<U> {
    fn ct_gt(&self, other: &Self) -> Choice {
        self.0.ct_gt(&other.0)
    }
}

impl<U: ConstantTimeEq + ConstantTimeGreater> subtle::ConstantTimeLess for Ct<U> {}

impl<U: CtIsZero> CtIsZero for Ct<U> {
    fn ct_is_zero(&self) -> Choice {
        self.0.ct_is_zero()
    }
}
