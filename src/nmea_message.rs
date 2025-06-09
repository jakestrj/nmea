use crate::nmea_frame::Frame;
use core::result::Result;
use core::result::Result::Err;
use err_derive::Error;
use fixed_queue::VecDeque;

pub const MAX_NMEA_PACKET_SIZE: usize = 223;

#[derive(PartialEq)]
enum MessageType {
    Single,
    Consecutive,
    Unknown,
}

#[derive(PartialEq)]
enum TransmissionType {
    Rx,
    Tx,
}

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error(display = "Message queue is empty")]
    EmptyQueue,
    #[error(display = "Queue is already full")]
    FullQueue,
    #[error(display = "Wrong transmission type")]
    TransmissionTypeMismatch,
    #[error(display = "Wrong sequence counter")]
    SequenceCountError,
    #[error(display = "Frame is out of sequence")]
    SequenceMismatch,
}

pub struct Message {
    queue: VecDeque<Frame, 31>,
    message_type: MessageType,
    transmission_type: TransmissionType,
    pub num_frames: u8,
    pub data_len: u8,
    pub sequence_counter: u8,
    cur_frame_counter: u8,
}

impl Message {
    pub fn new() -> Self {
        let queue = VecDeque::new();
        Self {
            queue,
            message_type: MessageType::Unknown,
            num_frames: 0,
            transmission_type: TransmissionType::Rx,
            data_len: 0,
            sequence_counter: 0,
            cur_frame_counter: 0,
        }
    }

    pub fn add_frame(&mut self, payload: &[u8; 8]) -> Result<bool, Error> {
        if self.transmission_type == TransmissionType::Tx {
            return Err(Error::TransmissionTypeMismatch);
        }
        if !self.queue.is_empty() && self.queue.len() as u8 == self.num_frames {
            return Err(Error::FullQueue);
        }
        let frame = Frame::from_bytes(payload);
        if frame.is_first_frame() {
            if frame.data_len().unwrap() <= 6 {
                self.num_frames = 1
            } else {
                self.num_frames = num_integer::div_floor(frame.data_len().unwrap(), 7) + 1;
            }
            self.sequence_counter = frame.sequence_counter();
            self.data_len = frame.data_len().unwrap();
            self.queue.push_back(frame);
            self.cur_frame_counter = 0;
        } else {
            if self.sequence_counter != frame.sequence_counter() {
                return Err(Error::SequenceCountError);
            }
            if self.cur_frame_counter + 1 != frame.frame_counter() {
                return Err(Error::SequenceMismatch);
            }
            let frame_counter = frame.frame_counter();
            if frame_counter >= self.num_frames - 1 {
                self.queue.push_back(frame);
                return Ok(true);
            } else {
                self.queue.push_back(frame);
            }
            self.cur_frame_counter = frame_counter;
        }
        return Ok(false);
    }

    pub fn from_payload(payload: &[u8], sequence_counter: u8) -> Self {
        let mut queue = VecDeque::new();
        if payload.len() <= 6 {
            let mut padded_payload: [u8; 6] = [0xFF; 6];
            padded_payload[..payload.len()].copy_from_slice(payload);
            let first_frame =
                Frame::first_frame(&padded_payload, payload.len() as u8, sequence_counter);
            let _ = queue.push_back(first_frame);
            // We can contain in a single frame.
            return Self {
                queue,
                message_type: MessageType::Single,
                num_frames: 1,
                transmission_type: TransmissionType::Tx,
                data_len: payload.len() as u8,
                sequence_counter: 0,
                cur_frame_counter: 0,
            };
        }
        // Process first frame.
        let first_frame = Frame::first_frame(
            payload[..6].try_into().unwrap(),
            payload.len() as u8,
            sequence_counter,
        );
        let _ = queue.push_back(first_frame);

        // Process consecutive frames.
        let num_chunks: u8 = num_integer::div_floor(payload.len() as u8 - 6, 7);
        let remaining_bytes = (payload.len() as u8 - 6) - 7 * num_chunks;
        let mut frame_counter: u8 = 1; // First frame is already processed.
        for i in 0..num_chunks {
            let frame = Frame::consecutive_frame(
                &payload[6 + (i as usize) * 7..6 + (i as usize) * 7 + 7]
                    .try_into()
                    .unwrap(),
                sequence_counter,
                frame_counter,
            );
            let _ = match frame {
                Ok(f) => queue.push_back(f),
                Err(_e) => panic!("Error creating consecutive frame"),
            };
            frame_counter += 1;
        }

        // Process last consecutive frame if not frame aligned.
        if remaining_bytes > 0 {
            let mut padded_payload: [u8; 7] = [0xFF; 7];
            for i in 0..remaining_bytes {
                padded_payload[i as usize] = payload[6 + (num_chunks as usize) * 7 + (i as usize)];
            }
            let last_frame =
                Frame::consecutive_frame(&padded_payload, sequence_counter, frame_counter);
            let _ = match last_frame {
                Ok(f) => queue.push_back(f),
                Err(_e) => panic!("Error creating last consecutive frame"),
            };
        }
        return Self {
            queue,
            message_type: MessageType::Consecutive,
            num_frames: 0,
            transmission_type: TransmissionType::Tx,
            data_len: payload.len() as u8,
            sequence_counter,
            cur_frame_counter: 0,
        };
    }

    pub fn pop_frame(&mut self) -> Option<Frame> {
        self.queue.pop_front()
    }

    pub fn get_payload(&mut self, buf: &mut [u8]) -> usize {
        buf.fill(0xFF);
        let mut i = 0;
        while !self.queue.is_empty() {
            let frame = self.pop_frame().unwrap();
            if frame.is_first_frame() {
                buf[0..6].copy_from_slice(frame.payload());
                i += 6
            } else {
                buf[i..i + 7].copy_from_slice(frame.payload());
                i += 7
            }
        }
        return self.data_len as usize;
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.message_type = MessageType::Unknown;
        self.transmission_type = TransmissionType::Rx;
        self.num_frames = 0;
        self.data_len = 0;
        self.sequence_counter = 0;
        self.cur_frame_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rx() {
        let mut msg = Message::new();
        let buf_1: [u8; 8] = [0x00, 0x19, 0x12, 0x7C, 0xEA, 0xD5, 0x12, 0x3D];
        let buf_2: [u8; 8] = [0x01, 0x31, 0xF3, 0xD0, 0xAC, 0xF2, 0x23, 0x1A];
        let buf_3: [u8; 8] = [0x02, 0x03, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00];
        let buf_4: [u8; 8] = [0x03, 0x20, 0xFF, 0xFF, 0x00, 0x70, 0xFF, 0xFF];
        assert_eq!(msg.add_frame(&buf_1).unwrap(), false);
        assert_eq!(msg.add_frame(&buf_2).unwrap(), false);
        assert_eq!(msg.add_frame(&buf_3).unwrap(), false);
        assert_eq!(msg.add_frame(&buf_4).unwrap(), true);
        assert_eq!(msg.num_frames, 4);
        assert_eq!(msg.sequence_counter, 0);

        let error_kind: Error = msg.add_frame(&buf_1).unwrap_err();
        assert_eq!(error_kind, Error::FullQueue);

        let mut buf: [u8; 223] = [0xFF; 223];
        msg.get_payload(&mut buf);
        let expected_payload: [u8; 25] = [
            0x12, 0x7C, 0xEA, 0xD5, 0x12, 0x3D, 0x31, 0xF3, 0xD0, 0xAC, 0xF2, 0x23, 0x1A, 0x03,
            0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x20, 0xFF, 0xFF, 0x00, 0x70,
        ];
        assert_eq!(buf[..25], expected_payload);
    }

    #[test]
    fn test_tx() {
        // Length 25 packet. Adds 2 bytes of padding to end.
        let received_packet: [u8; 25] = [
            0x12, 0x7C, 0xEA, 0xD5, 0x12, 0x3D, 0x31, 0xF3, 0xD0, 0xAC, 0xF2, 0x23, 0x1A, 0x03,
            0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x20, 0xFF, 0xFF, 0x00, 0x70,
        ];
        let mut msg = Message::from_payload(&received_packet, 0);
        let buf_1: [u8; 8] = [0x00, 0x19, 0x12, 0x7C, 0xEA, 0xD5, 0x12, 0x3D];
        let buf_2: [u8; 8] = [0x01, 0x31, 0xF3, 0xD0, 0xAC, 0xF2, 0x23, 0x1A];
        let buf_3: [u8; 8] = [0x02, 0x03, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00];
        let buf_4: [u8; 8] = [0x03, 0x20, 0xFF, 0xFF, 0x00, 0x70, 0xFF, 0xFF];
        assert_eq!(msg.pop_frame().unwrap().bytes, buf_1);
        assert_eq!(msg.pop_frame().unwrap().bytes, buf_2);
        assert_eq!(msg.pop_frame().unwrap().bytes, buf_3);
        assert_eq!(msg.pop_frame().unwrap().bytes, buf_4);
    }
}
