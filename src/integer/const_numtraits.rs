// const-num-traits 0.2 impls for Uint<N, B, OM>.
//
// Compiled unconditionally — sits alongside the existing num-traits impls in
// numtraits.rs without touching them.
//
// Personality: always Nct. The subtle/CT trait impls are also provided so that
// modmath's CT path (Field<T, Ct>::exp) compiles; the actual operations are
// vartime despite implementing the CT interface.

use crate::{Integer, Uint};
use const_num_traits::ops::byte_slice::{ByteSliceError, ByteSliceErrorKind};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeLess};

// ── Identity values ──────────────────────────────────────────────────────────

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::Zero for Uint<N, B, OM> {
    fn zero() -> Self {
        Self::ZERO
    }
    fn is_zero(&self) -> bool {
        Integer::is_zero(self)
    }
    fn set_zero(&mut self) {
        *self = Self::ZERO;
    }
}

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::One for Uint<N, B, OM> {
    fn one() -> Self {
        Self::ONE
    }
    fn is_one(&self) -> bool {
        *self == Self::ONE
    }
    fn set_one(&mut self) {
        *self = Self::ONE;
    }
}

// ── Personality ───────────────────────────────────────────────────────────────

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::HasPersonality
    for Uint<N, B, OM>
{
    type P = const_num_traits::Nct;
}

// ── Wrapping arithmetic ───────────────────────────────────────────────────────

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::ops::wrapping::WrappingAdd
    for Uint<N, B, OM>
{
    type Output = Self;
    fn wrapping_add(self, v: Self) -> Self {
        Integer::wrapping_add(self, v)
    }
}

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::ops::wrapping::WrappingSub
    for Uint<N, B, OM>
{
    type Output = Self;
    fn wrapping_sub(self, v: Self) -> Self {
        Integer::wrapping_sub(self, v)
    }
}

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::ops::wrapping::WrappingMul
    for Uint<N, B, OM>
{
    type Output = Self;
    fn wrapping_mul(self, v: Self) -> Self {
        Integer::wrapping_mul(self, v)
    }
}

// ── Overflowing arithmetic ────────────────────────────────────────────────────

impl<const N: usize, const B: usize, const OM: u8>
    const_num_traits::ops::overflowing::OverflowingAdd for Uint<N, B, OM>
{
    type Output = Self;
    fn overflowing_add(self, v: Self) -> (Self, bool) {
        Integer::overflowing_add(self, v)
    }
}

// ── Carrying / borrowing arithmetic ──────────────────────────────────────────

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::BorrowingSub
    for Uint<N, B, OM>
{
    type Output = Self;
    fn borrowing_sub(self, rhs: Self, borrow: bool) -> (Self, bool) {
        Integer::borrowing_sub(self, rhs, borrow)
    }
}

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::CarryingMul
    for Uint<N, B, OM>
{
    type Unsigned = Self;
    type Output = Self;

    fn carrying_mul(self, rhs: Self, carry: Self) -> (Self, Self) {
        Uint::carrying_mul(self, rhs, carry)
    }

    fn carrying_mul_add(self, rhs: Self, carry: Self, add: Self) -> (Self, Self) {
        Uint::carrying_mul_add(self, rhs, carry, add)
    }
}

// ── Parity ────────────────────────────────────────────────────────────────────

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::Parity for Uint<N, B, OM> {
    fn is_odd(self) -> bool {
        self.bytes[0] & 1 == 1
    }
    fn is_even(self) -> bool {
        self.bytes[0] & 1 == 0
    }
}

// ── ConstZero / ConstOne (required by PrimBits) ───────────────────────────────

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::ConstZero for Uint<N, B, OM> {
    const ZERO: Self = Self::ZERO;
}

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::ConstOne for Uint<N, B, OM> {
    const ONE: Self = Self::ONE;
}

// ── PrimBits ─────────────────────────────────────────────────────────────────

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::PrimBits for Uint<N, B, OM> {
    fn count_ones(self) -> u32 {
        Integer::count_ones(self)
    }

    fn count_zeros(self) -> u32 {
        Integer::count_zeros(self)
    }

    fn leading_zeros(self) -> u32 {
        Integer::leading_zeros(self)
    }

    fn trailing_zeros(self) -> u32 {
        Integer::trailing_zeros(self)
    }

    fn rotate_left(self, _n: u32) -> Self {
        todo!()
    }
    fn rotate_right(self, _n: u32) -> Self {
        todo!()
    }
    fn signed_shl(self, _n: u32) -> Self {
        todo!()
    }
    fn signed_shr(self, _n: u32) -> Self {
        todo!()
    }
    fn unsigned_shl(self, n: u32) -> Self {
        Integer::wrapping_shl(self, n)
    }
    fn unsigned_shr(self, n: u32) -> Self {
        Integer::wrapping_shr(self, n)
    }
    fn swap_bytes(self) -> Self {
        todo!()
    }
    fn from_be(x: Self) -> Self {
        todo!("{}", core::mem::size_of_val(&x))
    }
    fn from_le(x: Self) -> Self {
        todo!("{}", core::mem::size_of_val(&x))
    }
    fn to_be(self) -> Self {
        todo!()
    }
    fn to_le(self) -> Self {
        todo!()
    }
}

// ── BytesHolder ───────────────────────────────────────────────────────────────
//
// Newtype over [u8; N] that implements Default for all N (unlike [u8; N] which
// only has Default for N ≤ 32). rsa_heapless 0.4 FixedWidthUnsignedInt requires
// `type Bytes: NumBytes + Default + AsMut<[u8]>`.

#[derive(Clone, Copy)]
pub struct BytesHolder<const N: usize>(pub(crate) [u8; N]);

impl<const N: usize> Default for BytesHolder<N> {
    fn default() -> Self {
        Self([0u8; N])
    }
}
impl<const N: usize> AsRef<[u8]> for BytesHolder<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl<const N: usize> AsMut<[u8]> for BytesHolder<N> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
impl<const N: usize> core::borrow::Borrow<[u8]> for BytesHolder<N> {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}
impl<const N: usize> core::borrow::BorrowMut<[u8]> for BytesHolder<N> {
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
impl<const N: usize> PartialEq for BytesHolder<N> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<const N: usize> Eq for BytesHolder<N> {}
impl<const N: usize> PartialOrd for BytesHolder<N> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<const N: usize> Ord for BytesHolder<N> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl<const N: usize> core::hash::Hash for BytesHolder<N> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl<const N: usize> core::fmt::Debug for BytesHolder<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BytesHolder(")?;
        for b in &self.0 {
            write!(f, "{b:02x}")?;
        }
        write!(f, ")")
    }
}
#[cfg(feature = "zeroize")]
impl<const N: usize> zeroize::DefaultIsZeroes for BytesHolder<N> {}

// ── ToBytes / FromBytes ───────────────────────────────────────────────────────
//
// type Bytes = BytesHolder<N> — bnum stores bytes internally as [u8; N] (LE order).
// BytesHolder wraps [u8; N] with Default for all N so FixedWidthUnsignedInt
// compiles for N = 128/256 (RSA-1024/2048). ToBytes::Bytes == FromBytes::Bytes
// satisfies rsa_heapless's NumToBytes<Bytes = <T as NumFromBytes>::Bytes> bound.

// bnum stores bytes LE internally (index 0 = LSB). to_bytes() returns the raw
// [u8; N] in that LE layout; we reverse for BE without calling the B=0-only
// inherent to_be_bytes/from_be_bytes.

#[inline]
fn reverse_bytes<const N: usize>(mut arr: [u8; N]) -> [u8; N] {
    let mut i = 0;
    while i < N / 2 {
        let j = N - 1 - i;
        arr.swap(i, j);
        i += 1;
    }
    arr
}

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::ToBytes for Uint<N, B, OM> {
    type Bytes = BytesHolder<N>;

    fn to_be_bytes(self) -> BytesHolder<N> {
        BytesHolder(reverse_bytes(Integer::to_bytes(self)))
    }

    fn to_le_bytes(self) -> BytesHolder<N> {
        BytesHolder(Integer::to_bytes(self))
    }
}

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::ToBytes for &Uint<N, B, OM> {
    type Bytes = BytesHolder<N>;

    fn to_be_bytes(self) -> BytesHolder<N> {
        BytesHolder(reverse_bytes(Integer::to_bytes(*self)))
    }

    fn to_le_bytes(self) -> BytesHolder<N> {
        BytesHolder(Integer::to_bytes(*self))
    }
}

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::FromBytes for Uint<N, B, OM> {
    type Bytes = BytesHolder<N>;

    fn from_be_bytes(bytes: &BytesHolder<N>) -> Self {
        Uint::from_bytes(reverse_bytes(bytes.0))
    }

    fn from_le_bytes(bytes: &BytesHolder<N>) -> Self {
        Uint::from_bytes(bytes.0)
    }
}

// ── FromByteSlice ─────────────────────────────────────────────────────────────

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::FromByteSlice
    for Uint<N, B, OM>
{
    fn from_be_slice(bytes: &[u8]) -> Result<Self, ByteSliceError> {
        if bytes.is_empty() {
            return Err(ByteSliceError {
                kind: ByteSliceErrorKind::Empty,
            });
        }
        if bytes.len() > N {
            return Err(ByteSliceError {
                kind: ByteSliceErrorKind::Overflow,
            });
        }
        let mut buf = [0u8; N];
        let offset = N - bytes.len();
        let mut i = 0;
        while i < bytes.len() {
            buf[offset + i] = bytes[i];
            i += 1;
        }
        Ok(Uint::from_bytes(reverse_bytes(buf)))
    }

    fn from_le_slice(bytes: &[u8]) -> Result<Self, ByteSliceError> {
        if bytes.is_empty() {
            return Err(ByteSliceError {
                kind: ByteSliceErrorKind::Empty,
            });
        }
        if bytes.len() > N {
            return Err(ByteSliceError {
                kind: ByteSliceErrorKind::Overflow,
            });
        }
        let mut buf = [0u8; N];
        let mut i = 0;
        while i < bytes.len() {
            buf[i] = bytes[i];
            i += 1;
        }
        Ok(Uint::from_bytes(buf))
    }
}

// ── &Uint wrapping ops (needed by ed25519 verify for<'a> &'a T bounds) ───────

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::ops::wrapping::WrappingAdd
    for &Uint<N, B, OM>
{
    type Output = Uint<N, B, OM>;

    fn wrapping_add(self, v: Self) -> Uint<N, B, OM> {
        Integer::wrapping_add(*self, *v)
    }
}

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::ops::wrapping::WrappingSub
    for &Uint<N, B, OM>
{
    type Output = Uint<N, B, OM>;

    fn wrapping_sub(self, v: Self) -> Uint<N, B, OM> {
        Integer::wrapping_sub(*self, *v)
    }
}

// ── CT trait impls (vartime bodies, Ct-interface for compilation) ─────────────

impl<const N: usize, const B: usize, const OM: u8> ConstantTimeEq for Uint<N, B, OM> {
    fn ct_eq(&self, other: &Self) -> Choice {
        let mut diff = 0u8;
        let mut i = 0;
        while i < N {
            diff |= self.bytes[i] ^ other.bytes[i];
            i += 1;
        }
        Choice::from((diff == 0) as u8)
    }
}

impl<const N: usize, const B: usize, const OM: u8> ConditionallySelectable for Uint<N, B, OM> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut bytes = a.bytes;
        let mask = if choice.unwrap_u8() != 0 { 0xFFu8 } else { 0u8 };
        let mut i = 0;
        while i < N {
            bytes[i] = (a.bytes[i] & !mask) | (b.bytes[i] & mask);
            i += 1;
        }
        Self::from_bytes(bytes)
    }
}

// big-endian byte-by-byte comparison, most-significant first
impl<const N: usize, const B: usize, const OM: u8> subtle::ConstantTimeGreater for Uint<N, B, OM> {
    fn ct_gt(&self, other: &Self) -> Choice {
        let a = self.bytes;
        let b = other.bytes;
        let mut gt = 0u8;
        let mut lt = 0u8;
        let mut i = N;
        while i > 0 {
            i -= 1;
            // u16 subtraction so the borrow bit (bit 15) correctly captures
            // "b[i] < a[i]" for all unsigned byte values. u8 >> 7 fails when
            // the difference wraps into [0,127] (i.e. a[i]-b[i] > 128).
            gt |= (((b[i] as u16).wrapping_sub(a[i] as u16) >> 15) as u8) & !(gt | lt);
            lt |= (((a[i] as u16).wrapping_sub(b[i] as u16) >> 15) as u8) & !(gt | lt);
        }
        Choice::from(gt)
    }
}

impl<const N: usize, const B: usize, const OM: u8> ConstantTimeLess for Uint<N, B, OM> {}

impl<const N: usize, const B: usize, const OM: u8> const_num_traits::CtIsZero for Uint<N, B, OM> {
    fn ct_is_zero(&self) -> Choice {
        self.ct_eq(&Self::ZERO)
    }
}
