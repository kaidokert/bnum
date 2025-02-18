
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BytesHolder<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> Default for BytesHolder<N> {
    fn default() -> Self {
        Self { bytes: [0; N] }
    }
}

impl<const N: usize> core::borrow::Borrow<[u8]> for BytesHolder<N> {
    fn borrow(&self) -> &[u8] {
        &self.bytes
    }
}
impl<const N: usize> core::borrow::BorrowMut<[u8]> for BytesHolder<N> {
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}
impl<const N: usize> AsRef<[u8]> for BytesHolder<N> {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}
impl<const N: usize> AsMut<[u8]> for BytesHolder<N> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

