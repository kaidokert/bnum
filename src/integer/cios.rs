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

#[cfg(test)]
mod montgomery_tests {
    use super::*;
    use modmath_cios::CiosRowOps;

    // Curve25519 prime: 2^255 - 19
    // as little-endian bytes
    const P25519_LE: [u8; 32] = [
        0xed, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f,
    ];

    fn u256_from_le(bytes: [u8; 32]) -> Uint<32> {
        Uint::<32>::from_le_bytes(bytes)
    }

    // Simple (slow) reference: a*b mod m
    fn ref_mulmod(a: u128, b: u128, m: u128) -> u128 {
        // Use 256-bit intermediates via u128 splitting for small values only
        ((a as u128 * b as u128) % m as u128)
    }

    // Test mul_acc_row matches direct u64 widening multiply for single-limb
    #[test]
    fn mul_acc_row_matches_u64() {
        let scalar: u64 = 0xDEADBEEFCAFEBABE;
        let mult_val: u64 = 0x0123456789ABCDEF;

        // Direct u128 multiply
        let expected = scalar as u128 * mult_val as u128;

        let mut mult = Uint::<8>::ZERO;
        write_u64(mult.as_bytes_mut(), 0, mult_val);
        let mut acc = Uint::<8>::ZERO;
        let carry = <Uint<8> as CiosRowOps>::mul_acc_row(scalar, &mult, &mut acc, 0);

        let got_lo = read_u64(acc.as_bytes(), 0);
        let got_hi = carry;
        let got = got_lo as u128 | ((got_hi as u128) << 64);
        assert_eq!(got, expected, "mul_acc_row wrong: {got:#034x} != {expected:#034x}");
    }

    // Test that Montgomery multiplication identity: mont_mul(a_mont, 1_mont) = a_mont
    #[test]
    fn mont_mul_identity_small_modulus() {
        // Use modulus = 13, R = 2^64 (1 limb)
        type U8 = Uint<8>;
        let m: u64 = 13;
        // n' = -m^-1 mod 2^64
        // m * m_inv ≡ 1 (mod 2^64) → use extended Euclidean or Newton's method
        // For m=13: 13 * x ≡ 1 (mod 2^64)
        // Newton: x0 = 1, x_{i+1} = x_i * (2 - m * x_i) mod 2^64
        let mut x: u64 = 1u64;
        let mut i = 0;
        while i < 6 { x = x.wrapping_mul(2u64.wrapping_sub(m.wrapping_mul(x))); i += 1; }
        let n_prime = x.wrapping_neg(); // -m^-1 mod 2^64
        // verify: m * n_prime ≡ -1 (mod 2^64)
        assert_eq!(m.wrapping_mul(n_prime).wrapping_add(1), 0, "n_prime wrong");

        // R = 2^64, R mod m = 2^64 mod 13
        // 2^64 mod 13: 2^1=2, 2^2=4, 2^3=8, 2^4=3, 2^5=6, 2^6=12, 2^7=11, 2^8=9, 2^9=5, 2^10=10, 2^11=7, 2^12=1 → period 12
        // 64 mod 12 = 4 → 2^64 mod 13 = 2^4 mod 13 = 3
        let r_mod_m: u64 = 3;

        // a = 5 → a_mont = a * R mod m = 5*3 mod 13 = 15 mod 13 = 2
        let a: u64 = 5;
        let a_mont: u64 = (a * r_mod_m) % m;
        // 1_mont = R mod m = 3
        let one_mont: u64 = r_mod_m;

        let a_big = U8::from_le_bytes({let mut b = [0u8;8]; let arr = a_mont.to_le_bytes(); let mut j=0; while j<8{b[j]=arr[j];j+=1;} b});
        let one_big = U8::from_le_bytes({let mut b = [0u8;8]; let arr = one_mont.to_le_bytes(); let mut j=0; while j<8{b[j]=arr[j];j+=1;} b});
        let m_big = U8::from_le_bytes({let mut b = [0u8;8]; let arr = m.to_le_bytes(); let mut j=0; while j<8{b[j]=arr[j];j+=1;} b});
        let np_big = U8::from_le_bytes({let mut b = [0u8;8]; let arr = n_prime.to_le_bytes(); let mut j=0; while j<8{b[j]=arr[j];j+=1;} b});

        // mont_mul(a_mont, 1_mont) should equal a_mont
        let result = {
            let n = 1usize; // word count
            let zero = 0u64;
            let one_w = 1u64;
            let mut acc = U8::ZERO;
            let mut acc_hi = zero;
            let mut acc_hi2 = zero;
            let mut i = 0;
            while i < n {
                let ai = CiosRowOps::word(&a_big, i);
                let carry = <U8 as CiosRowOps>::mul_acc_row(ai, &one_big, &mut acc, zero);
                let (sum, overflow) = acc_hi.overflowing_add(carry);
                acc_hi = sum;
                if overflow { acc_hi2 = acc_hi2 + one_w; }
                let m_w = CiosRowOps::word(&acc, 0).wrapping_mul(CiosRowOps::word(&np_big, 0));
                let new_overflow = <U8 as CiosRowOps>::mul_acc_shift_row(m_w, &m_big, &mut acc, acc_hi);
                acc_hi = acc_hi2 + new_overflow;
                acc_hi2 = zero;
                i += 1;
            }
            if acc_hi > zero || acc >= m_big { 
                let (r, _) = acc.overflowing_sub(m_big); acc = r;
            }
            acc
        };
        assert_eq!(result, a_big, "mont_mul identity failed: got {:?} != {:?}", result, a_big);
    }

    #[test]
    fn mont_mul_p25519_small_values() {
        type U256 = Uint<32>;
        let p_bytes: [u8; 32] = [
            0xed, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f,
        ];
        let p = U256::from_le_bytes(p_bytes);
        let p0 = read_u64(&p_bytes, 0);
        let mut np: u64 = 1;
        let mut k = 0;
        while k < 6 { np = np.wrapping_mul(2u64.wrapping_sub(p0.wrapping_mul(np))); k += 1; }
        let n_prime_0 = np.wrapping_neg();
        let np_u256 = {
            let mut b = [0u8; 32]; let a = n_prime_0.to_le_bytes();
            let mut j = 0; while j < 8 { b[j] = a[j]; j += 1; } U256::from_le_bytes(b)
        };
        let mk = |v: u64| -> U256 {
            let mut b = [0u8; 32]; let a = v.to_le_bytes();
            let mut j = 0; while j < 8 { b[j] = a[j]; j += 1; } U256::from_le_bytes(b)
        };
        // R = 2^256 ≡ 38 mod p (since 2^256 = 2*(2^255) = 2*(p+19) = 2p+38 ≡ 38 mod p)
        let r_mod_p: u64 = 38;
        let two_m = mk(2 * r_mod_p);
        let three_m = mk(3 * r_mod_p);

        let zero = 0u64;
        let one_w = 1u64;
        let mut acc = U256::ZERO;
        let mut acc_hi = zero;
        let mut acc_hi2 = zero;
        let n = CiosRowOps::word_count(&p);
        let mut i = 0;
        while i < n {
            let ai = CiosRowOps::word(&two_m, i);
            let carry = <U256 as CiosRowOps>::mul_acc_row(ai, &three_m, &mut acc, zero);
            let (sum, overflow) = acc_hi.overflowing_add(carry);
            acc_hi = sum;
            if overflow { acc_hi2 = acc_hi2 + one_w; }
            let m_w = CiosRowOps::word(&acc, 0).wrapping_mul(CiosRowOps::word(&np_u256, 0));
            let new_overflow = <U256 as CiosRowOps>::mul_acc_shift_row(m_w, &p, &mut acc, acc_hi);
            acc_hi = acc_hi2 + new_overflow;
            acc_hi2 = zero;
            i += 1;
        }
        if acc_hi > zero || acc >= p { let (r, _) = acc.overflowing_sub(p); acc = r; }

        // mont_mul(2R, 3R) = 6 * R^2 * R^{-1} mod p = 6*R mod p = 6*38 = 228
        let expected = mk(6 * r_mod_p);
        assert_eq!(acc, expected,
            "2*3 Montgomery mod p25519 wrong: got word0={:#018x} expected word0={:#018x}",
            read_u64(acc.as_bytes(), 0), read_u64(expected.as_bytes(), 0));
    }
}
