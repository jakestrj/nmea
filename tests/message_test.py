import math
import random

import pytest
from assertpy import assert_that
from nmea import Message as NmeaMessage


def test_rx():
    msg = NmeaMessage()

    buf_1 = bytes([0x00, 0x19, 0x12, 0x7C, 0xEA, 0xD5, 0x12, 0x3D])
    buf_2 = bytes([0x01, 0x31, 0xF3, 0xD0, 0xAC, 0xF2, 0x23, 0x1A])
    buf_3 = bytes([0x02, 0x03, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00])
    buf_4 = bytes([0x03, 0x20, 0xFF, 0xFF, 0x00, 0x70, 0xFF, 0xFF])

    assert_that(msg.add_frame(buf_1)).is_false()
    assert_that(msg.add_frame(buf_2)).is_false()
    assert_that(msg.add_frame(buf_3)).is_false()
    assert_that(msg.add_frame(buf_4)).is_true()

    assert_that(msg.num_frames).is_equal_to(4)
    assert_that(msg.sequence_counter).is_equal_to(0)

    with pytest.raises(Exception) as exc_info:
        msg.add_frame(buf_1)
    assert_that(str(exc_info.value)).is_equal_to("Queue is already full")

    buf = msg.get_payload()

    expected_payload = bytes(
        [
            0x12,
            0x7C,
            0xEA,
            0xD5,
            0x12,
            0x3D,
            0x31,
            0xF3,
            0xD0,
            0xAC,
            0xF2,
            0x23,
            0x1A,
            0x03,
            0xFF,
            0xFF,
            0x00,
            0x00,
            0x00,
            0x00,
            0x20,
            0xFF,
            0xFF,
            0x00,
            0x70,
        ]
    )

    assert_that(buf[:25]).is_equal_to(expected_payload)


def test_tx():
    received_packet = bytes(
        [
            0x12,
            0x7C,
            0xEA,
            0xD5,
            0x12,
            0x3D,
            0x31,
            0xF3,
            0xD0,
            0xAC,
            0xF2,
            0x23,
            0x1A,
            0x03,
            0xFF,
            0xFF,
            0x00,
            0x00,
            0x00,
            0x00,
            0x20,
            0xFF,
            0xFF,
            0x00,
            0x70,
        ]
    )

    msg = NmeaMessage.from_payload(received_packet, 0)

    buf_1 = bytes([0x00, 0x19, 0x12, 0x7C, 0xEA, 0xD5, 0x12, 0x3D])
    buf_2 = bytes([0x01, 0x31, 0xF3, 0xD0, 0xAC, 0xF2, 0x23, 0x1A])
    buf_3 = bytes([0x02, 0x03, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00])
    buf_4 = bytes([0x03, 0x20, 0xFF, 0xFF, 0x00, 0x70, 0xFF, 0xFF])

    assert_that(msg).is_not_none()
    assert_that(msg.pop_frame()).is_equal_to(buf_1)
    assert_that(msg.pop_frame()).is_equal_to(buf_2)
    assert_that(msg.pop_frame()).is_equal_to(buf_3)
    assert_that(msg.pop_frame()).is_equal_to(buf_4)
