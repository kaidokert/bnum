// CiosRowOps for Uint<N, B, OM>.
//
// Bytes stored little-endian; u64 limbs by reading/writing 8-byte slices.
// N must be a multiple of 8 — checked at first call to word_count.

use super::Uint;
use modmath_cios::CiosRowOps;

#[inline(always)]
const fn read_u64(bytes: &[u8], limb: usize) -> u64 {
    let s = limb * 8;
    u64::from_le_bytes([
        bytes[s], bytes[s+1], bytes[s+2], bytes[s+3],
        bytes[s+4], bytes[s+5], bytes[s+6], bytes[s+7],
    ])
}

#[inline(always)]
fn write_u64(bytes: &mut [u8], limb: usize, val: u64) {
    let s = limb * 8;
    let arr = val.to_le_bytes();
    let mut j = 0;
    while j < 8 {
        bytes[s + j] = arr[j];
        j += 1;
    }
}

impl<const N: usize, const B: usize, const OM: u8> CiosRowOps for Uint<N, B, OM> {
    type Word = u64;

    #[inline]
    fn word_count(&self) -> usize {
        const { assert!(N % 8 == 0, "Uint byte width N must be a multiple of 8 for CiosRowOps") }
        N / 8
    }

    #[inline]
    fn word(&self, i: usize) -> u64 {
        read_u64(self.as_bytes(), i)
    }

    fn mul_acc_row(scalar: u64, multiplicand: &Self, acc: &mut Self, carry_in: u64) -> u64 {
        let words = N / 8;
        let mut carry = carry_in as u128;
        let m = multiplicand.as_bytes().as_slice();
        let a = acc.as_bytes_mut().as_mut_slice();
        let mut j = 0;
        while j < words {
            let product = scalar as u128 * read_u64(m, j) as u128 + read_u64(a, j) as u128 + carry;
            write_u64(a, j, product as u64);
            carry = product >> 64;
            j += 1;
        }
        carry as u64
    }

    fn mul_acc_shift_row(scalar: u64, multiplicand: &Self, acc: &mut Self, acc_hi: u64) -> u64 {
        let words = N / 8;
        let m = multiplicand.as_bytes().as_slice();
        let a = acc.as_bytes_mut().as_mut_slice();

        // Word 0: discard the low word, keep carry.
        let p0 = scalar as u128 * read_u64(m, 0) as u128 + read_u64(a, 0) as u128;
        let mut carry = (p0 >> 64) as u64;

        // Words 1..words: shift down by one position.
        let mut j = 1;
        while j < words {
            let product = scalar as u128 * read_u64(m, j) as u128 + read_u64(a, j) as u128 + carry as u128;
            write_u64(a, j - 1, product as u64);
            carry = (product >> 64) as u64;
            j += 1;
        }

        // Fold acc_hi + carry into acc[words-1]; return overflow bit (0 or 1).
        let (sum, overflow) = acc_hi.overflowing_add(carry);
        write_u64(a, words - 1, sum);
        overflow as u64
    }
}
