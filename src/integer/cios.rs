// CiosRowOps implementation for Uint<N, B, OM>.
//
// Bytes are stored little-endian; we expose u64 limbs by reading/writing
// 8-byte slices.  N must be divisible by 8 (checked at monomorphization).
//
// Only Uint (unsigned) implements CiosRowOps — CIOS Montgomery multiplication
// is an unsigned operation; signed wrappers don't make sense here.

#![cfg(feature = "cios")]

use super::Uint;
use modmath_cios::CiosRowOps;

#[inline(always)]
fn read_u64(bytes: &[u8], limb_idx: usize) -> u64 {
    let start = limb_idx * 8;
    let mut arr = [0u8; 8];
    let mut j = 0;
    while j < 8 {
        arr[j] = bytes[start + j];
        j += 1;
    }
    u64::from_le_bytes(arr)
}

#[inline(always)]
fn write_u64(bytes: &mut [u8], limb_idx: usize, val: u64) {
    let start = limb_idx * 8;
    let arr = val.to_le_bytes();
    let mut j = 0;
    while j < 8 {
        bytes[start + j] = arr[j];
        j += 1;
    }
}

impl<const N: usize, const B: usize, const OM: u8> CiosRowOps for Uint<N, B, OM> {
    type Word = u64;

    fn word_count(&self) -> usize {
        const { assert!(N % 8 == 0, "Uint byte width N must be divisible by 8 for CiosRowOps") }
        N / 8
    }

    fn word(&self, i: usize) -> u64 {
        read_u64(self.as_bytes(), i)
    }

    fn mul_acc_row(
        scalar: u64,
        multiplicand: &Self,
        acc: &mut Self,
        carry_in: u64,
    ) -> u64 {
        let words = N / 8;
        let mut carry = carry_in as u128;
        let m_bytes = multiplicand.as_bytes();
        let a_bytes = acc.as_bytes_mut();
        let mut j = 0;
        while j < words {
            let m = read_u64(m_bytes, j);
            let a = read_u64(a_bytes, j);
            let product = scalar as u128 * m as u128 + a as u128 + carry;
            write_u64(a_bytes, j, product as u64);
            carry = product >> 64;
            j += 1;
        }
        carry as u64
    }

    fn mul_acc_shift_row(
        scalar: u64,
        multiplicand: &Self,
        acc: &mut Self,
        acc_hi: u64,
    ) -> u64 {
        let words = N / 8;
        let m_bytes = multiplicand.as_bytes();
        let a_bytes = acc.as_bytes_mut();

        // Word 0: compute and discard (zero by CIOS construction), keep carry.
        let m0 = read_u64(m_bytes, 0);
        let a0 = read_u64(a_bytes, 0);
        let p0 = scalar as u128 * m0 as u128 + a0 as u128;
        let mut carry = (p0 >> 64) as u64;

        // Words 1..words: compute and shift down by one position.
        let mut j = 1;
        while j < words {
            let m = read_u64(m_bytes, j);
            let a = read_u64(a_bytes, j);
            let product = scalar as u128 * m as u128 + a as u128 + carry as u128;
            write_u64(a_bytes, j - 1, product as u64);
            carry = (product >> 64) as u64;
            j += 1;
        }

        // Fold acc_hi + carry into acc[words-1]; return overflow bit (0 or 1).
        let (sum, overflow) = acc_hi.overflowing_add(carry);
        write_u64(a_bytes, words - 1, sum);
        overflow as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use modmath_cios::CiosRowOps;

    type U128 = Uint<16>;  // 128-bit: 2 u64 limbs
    type U256 = Uint<32>;  // 256-bit: 4 u64 limbs

    #[test]
    fn word_count_128() {
        let v = U128::ZERO;
        assert_eq!(CiosRowOps::word_count(&v), 2);
    }

    #[test]
    fn word_count_256() {
        let v = U256::ZERO;
        assert_eq!(CiosRowOps::word_count(&v), 4);
    }

    #[test]
    fn word_roundtrip() {
        // Build a 128-bit value with known limbs and verify word() reads them back.
        let mut v = U128::ZERO;
        let bytes = v.as_bytes_mut();
        // limb 0 = 0x1122334455667788, limb 1 = 0xAABBCCDDEEFF0011
        let lo: u64 = 0x1122334455667788;
        let hi: u64 = 0xAABBCCDDEEFF0011;
        write_u64(bytes, 0, lo);
        write_u64(bytes, 1, hi);
        assert_eq!(CiosRowOps::word(&v, 0), lo);
        assert_eq!(CiosRowOps::word(&v, 1), hi);
    }

    #[test]
    fn mul_acc_row_zero_scalar() {
        let mult = U128::ONE;
        let mut acc = U128::ONE;
        let carry = <U128 as CiosRowOps>::mul_acc_row(0, &mult, &mut acc, 0);
        assert_eq!(CiosRowOps::word(&acc, 0), 1);
        assert_eq!(carry, 0);
    }

    #[test]
    fn mul_acc_row_no_carry() {
        // 3 * 4 + 0 = 12, fits in low limb.
        let mut mult = U128::ZERO;
        write_u64(mult.as_bytes_mut(), 0, 4);
        let mut acc = U128::ZERO;
        let carry = <U128 as CiosRowOps>::mul_acc_row(3, &mult, &mut acc, 0);
        assert_eq!(CiosRowOps::word(&acc, 0), 12);
        assert_eq!(CiosRowOps::word(&acc, 1), 0);
        assert_eq!(carry, 0);
    }

    #[test]
    fn mul_acc_row_carry_into_hi_limb() {
        // scalar = u64::MAX, mult[0] = u64::MAX, mult[1] = 0, acc = 0, carry_in = 0
        // j=0: product = u64::MAX*u64::MAX = 0xFFFFFFFFFFFFFFFE_0000000000000001
        //      acc[0] = 1, carry = u64::MAX-1 (= 0xFFFFFFFFFFFFFFFE)
        // j=1: mult[1]=0, acc[1]=0: product = 0 + 0 + 0xFFFFFFFFFFFFFFFE
        //      acc[1] = 0xFFFFFFFFFFFFFFFE, carry = 0
        let mut mult = U128::ZERO;
        write_u64(mult.as_bytes_mut(), 0, u64::MAX);
        let mut acc = U128::ZERO;
        let carry = <U128 as CiosRowOps>::mul_acc_row(u64::MAX, &mult, &mut acc, 0);
        assert_eq!(CiosRowOps::word(&acc, 0), 1);
        assert_eq!(CiosRowOps::word(&acc, 1), u64::MAX - 1);
        assert_eq!(carry, 0);
    }

    #[test]
    fn mul_acc_shift_row_zero() {
        let mult = U128::ZERO;
        let mut acc = U128::ZERO;
        let overflow = <U128 as CiosRowOps>::mul_acc_shift_row(0, &mult, &mut acc, 0);
        assert_eq!(acc, U128::ZERO);
        assert_eq!(overflow, 0);
    }

    #[test]
    fn mul_acc_shift_row_matches_phase1_then_shift() {
        // Manual verification: scalar=1, mult=u64::MAX, acc=0, acc_hi=0
        // Phase 1 would produce: product = u64::MAX, carry = 0
        // Shift: acc[0] is discarded (=u64::MAX from product), acc[0] gets carry (0), acc_hi (0) folds in.
        let mut mult = U128::ZERO;
        write_u64(mult.as_bytes_mut(), 0, u64::MAX);
        let mut acc = U128::ZERO;
        let overflow = <U128 as CiosRowOps>::mul_acc_shift_row(1, &mult, &mut acc, 0);
        // low limb discarded, high limb = 0+carry(0) + acc_hi(0) = 0
        assert_eq!(CiosRowOps::word(&acc, 0), 0);
        assert_eq!(CiosRowOps::word(&acc, 1), 0);
        assert_eq!(overflow, 0);
    }

    #[test]
    fn matches_fixed_bigint_phase1_identity() {
        // primitive u64 impl from modmath-cios vs Uint<8> impl should agree on 1-limb values.
        let scalar: u64 = 0x123456789ABCDEF0;
        let mult_prim: u64 = 0x0FEDCBA987654321;
        let mut acc_prim: u64 = 0x1111111111111111;
        let carry_prim = <u64 as CiosRowOps>::mul_acc_row(scalar, &mult_prim, &mut acc_prim, 0);

        let mut mult_big = Uint::<8>::ZERO;
        write_u64(mult_big.as_bytes_mut(), 0, mult_prim);
        let mut acc_big = Uint::<8>::ZERO;
        write_u64(acc_big.as_bytes_mut(), 0, 0x1111111111111111u64);
        let carry_big = <Uint<8> as CiosRowOps>::mul_acc_row(scalar, &mult_big, &mut acc_big, 0);

        assert_eq!(CiosRowOps::word(&acc_big, 0), acc_prim);
        assert_eq!(carry_big, carry_prim);
    }
}
