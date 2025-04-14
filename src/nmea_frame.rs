use err_derive::Error;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Single,
    First,
    Consecutive,
    Flow,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowKind {
    Continue,
    Wait,
    Abort,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "Invalid input parameter")]
    InvalidParameter,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub bytes: [u8; 8],
}

impl Frame {
    pub fn first_frame(bytes: &[u8; 6], len: u8, sequence_counter: u8) -> Self {
        let mut buf: [u8; 8] = [0xFF; 8];
        buf[0] = sequence_counter << 5;
        buf[1] = len;
        buf[2..].copy_from_slice(bytes);
        Self { bytes: buf }
    }

    pub fn consecutive_frame(
        bytes: &[u8; 7],
        sequence_counter: u8,
        frame_counter: u8,
    ) -> Result<Self, Error> {
        let mut buf: [u8; 8] = [0xFF; 8];
        if sequence_counter > 7 || frame_counter > 31 {
            return Err(Error::InvalidParameter);
        }
        buf[0] = (sequence_counter << 5) | frame_counter;
        buf[1..].copy_from_slice(bytes);
        Ok(Self { bytes: buf })
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut buf = [0; 8];
        for (a, b) in bytes.iter().zip(&mut buf) {
            *b = *a;
        }

        Self { bytes: buf }
    }

    pub fn sequence_counter(&self) -> u8 {
        return (self.bytes[0] & 0xE0) >> 5;
    }

    pub fn frame_counter(&self) -> u8 {
        return self.bytes[0] & 0x1F;
    }

    pub fn data_len(&self) -> Option<u8> {
        if !self.is_first_frame() {
            // Not first frame
            return None;
        }
        Some(self.bytes[1])
    }

    pub fn payload(&self) -> &[u8] {
        if !self.is_first_frame() {
            // Not first frame
            return &self.bytes[1..];
        }
        &self.bytes[2..]
    }

    pub fn is_first_frame(&self) -> bool {
        return self.frame_counter() == 0;
    }
}

impl AsRef<[u8]> for Frame {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_frame() {
        let mut first_frame: [u8; 8] = [0x00, 0x1B, 0x12, 0x7C, 0xEA, 0xD5, 0x12, 0x3D];
        let test_frame = Frame::from_bytes(&first_frame);
        assert_eq!(test_frame.sequence_counter(), 0);
        assert_eq!(test_frame.frame_counter(), 0);
        assert_eq!(test_frame.data_len(), Some(27));
        assert_eq!(test_frame.payload(), [0x12, 0x7C, 0xEA, 0xD5, 0x12, 0x3D]);

        let first_frame_payload: [u8; 6] = [0x12, 0x7C, 0xEA, 0xD5, 0x12, 0x3D];
        let test_first_frame = Frame::first_frame(&first_frame_payload, 27, 0);
        assert_eq!(test_first_frame.sequence_counter(), 0);
        assert_eq!(test_first_frame.frame_counter(), 0);
        assert_eq!(test_first_frame.data_len(), Some(27));
        assert_eq!(
            test_first_frame.payload(),
            [0x12, 0x7C, 0xEA, 0xD5, 0x12, 0x3D]
        );
    }

    #[test]
    fn test_consecutive_frame() {
        let frame_counter = 3;
        let sequence_counter = 1;
        let mut payload: [u8; 7] = [0x20, 0xFF, 0xFF, 0x00, 0x70, 0xFE, 0xFF];
        let test_frame: Frame =
            Frame::consecutive_frame(&payload, sequence_counter, frame_counter).unwrap();
        assert_eq!(test_frame.sequence_counter(), 1);
        assert_eq!(test_frame.frame_counter(), 3);
        assert_eq!(test_frame.data_len(), None);
        assert_eq!(
            test_frame.payload(),
            [0x20, 0xFF, 0xFF, 0x00, 0x70, 0xFE, 0xFF]
        );
    }
}
