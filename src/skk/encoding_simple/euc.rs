use crate::skk::encoding_simple::*;

impl Euc {
    pub(crate) fn decode(euc_buffer: &[u8]) -> Result<Vec<u8>, SkkError> {
        if euc_buffer.is_empty() {
            return Ok(Vec::new());
        }
        let is_error_exit = false;
        let mut euc_i = 0;
        let mut result_utf8 = Vec::new();
        let euc_buffer_length = euc_buffer.len();
        let euc_2_to_utf8_vec = EUC_2_TO_UTF8_VEC.read().unwrap();
        let euc_3_to_utf8_map = EUC_3_TO_UTF8_MAP.read().unwrap();
        let combine_euc_to_utf8_map = COMBINE_EUC_TO_UTF8_MAP.read().unwrap();
        // HashMap へのアクセスが遅いので注意。
        // できるだけ HashMap に触れる前に弾くこと。
        loop {
            match euc_buffer[euc_i] {
                0xa1..=0xfe | 0x8e => {
                    if !Utility::is_enough_2_bytes(euc_buffer_length, euc_i) {
                        result_utf8.extend_from_slice(
                            &format!("&#x{:>02x}", euc_buffer[euc_i]).as_bytes(),
                        );
                        if is_error_exit {
                            return Err(SkkError::Encoding);
                        }
                        break;
                    }
                    let euc_2_index = Decoder::get_euc_2_to_utf8_vec_index(
                        euc_buffer[euc_i],
                        euc_buffer[euc_i + 1],
                    );
                    if Utility::contains_euc_2(&euc_2_to_utf8_vec, euc_2_index) {
                        Decoder::push_to_buffer_utf8(
                            &euc_2_to_utf8_vec[euc_2_index],
                            &mut result_utf8,
                        );
                    } else {
                        let array_key_3_in_2 = [euc_buffer[euc_i], euc_buffer[euc_i + 1], 0];
                        if combine_euc_to_utf8_map.contains_key(&array_key_3_in_2) {
                            let v = combine_euc_to_utf8_map[&array_key_3_in_2];
                            Decoder::push_to_buffer_utf8(&v[..4], &mut result_utf8);
                            Decoder::push_to_buffer_utf8(&v[4..8], &mut result_utf8);
                        } else {
                            result_utf8.extend_from_slice(
                                &format!(
                                    "&#x{:>02x}{:>02x}",
                                    array_key_3_in_2[0], array_key_3_in_2[1]
                                )
                                .as_bytes(),
                            );
                            if is_error_exit {
                                return Err(SkkError::Encoding);
                            }
                        }
                    }
                    euc_i += 2;
                }
                0x00..=0x7f => {
                    result_utf8.push(euc_buffer[euc_i]);
                    euc_i += 1;
                }
                0x8f => {
                    if !Utility::is_enough_3_bytes(euc_buffer_length, euc_i) {
                        if euc_i + 1 >= euc_buffer_length {
                            result_utf8.extend_from_slice(
                                &format!("&#x{:>02x}", euc_buffer[euc_i]).as_bytes(),
                            );
                        } else {
                            result_utf8.extend_from_slice(
                                &format!(
                                    "&#x{:>02x}{:>02x}",
                                    euc_buffer[euc_i],
                                    euc_buffer[euc_i + 1]
                                )
                                .as_bytes(),
                            );
                        }
                        if is_error_exit {
                            return Err(SkkError::Encoding);
                        }
                        break;
                    }
                    let key_3 = &euc_buffer[euc_i..euc_i + 3];
                    if euc_3_to_utf8_map.contains_key(key_3) {
                        Decoder::push_to_buffer_utf8(&euc_3_to_utf8_map[key_3], &mut result_utf8);
                    } else {
                        result_utf8.extend_from_slice(
                            &format!("&#x{:>02x}{:>02x}{:>02x}", key_3[0], key_3[1], key_3[2])
                                .as_bytes(),
                        );
                        if is_error_exit {
                            return Err(SkkError::Encoding);
                        }
                    }
                    euc_i += 3;
                }
                _ => {
                    result_utf8
                        .extend_from_slice(&format!("&#x{:>02x}", euc_buffer[euc_i]).as_bytes());
                    euc_i += 1;
                }
            }
            #[cfg(feature = "assert_paranoia")]
            {
                assert!(euc_i <= euc_buffer_length);
            }
            if euc_i >= euc_buffer_length {
                break;
            }
        }
        Ok(result_utf8)
    }

    pub(crate) fn encode(utf8_buffer: &[u8]) -> Result<Vec<u8>, SkkError> {
        if utf8_buffer.is_empty() {
            return Ok(Vec::new());
        }
        let is_error_exit = false;
        let mut utf8_i = 0;
        let mut result_euc = Vec::new();
        let utf8_buffer_length = utf8_buffer.len();
        let utf8_3_to_euc_vec = UTF8_3_TO_EUC_VEC.read().unwrap();
        let utf8_2_4_to_euc_map = UTF8_2_4_TO_EUC_MAP.read().unwrap();
        let combine_utf8_4_to_euc_map = COMBINE_UTF8_4_TO_EUC_MAP.read().unwrap();
        let combine_utf8_6_to_euc_map = COMBINE_UTF8_6_TO_EUC_MAP.read().unwrap();
        // HashMap へのアクセスが遅いので注意。
        // できるだけ Vec や is_candidate_combine_*() などで HashMap に触れる前に弾くこと。
        loop {
            match utf8_buffer[utf8_i] {
                0xe0..=0xef
                    if Encoder::is_utf8_3_bytes(&utf8_buffer, utf8_buffer_length, utf8_i) =>
                {
                    if Utility::is_enough_6_bytes(utf8_buffer_length, utf8_i)
                        && Encoder::is_candidate_combine_utf8_6(utf8_buffer, utf8_i)
                        && combine_utf8_6_to_euc_map.contains_key(&utf8_buffer[utf8_i..utf8_i + 6])
                    {
                        Encoder::push_to_buffer_euc(
                            &combine_utf8_6_to_euc_map[&utf8_buffer[utf8_i..utf8_i + 6]],
                            &mut result_euc,
                        );
                        utf8_i += 3 + 3;
                    } else {
                        let utf8_3_index =
                            Encoder::get_utf8_3_to_euc_vec_index(&utf8_buffer[utf8_i..utf8_i + 3]);
                        if Utility::contains_utf8_3(&utf8_3_to_euc_vec, utf8_3_index) {
                            Encoder::push_to_buffer_euc(
                                &utf8_3_to_euc_vec[utf8_3_index],
                                &mut result_euc,
                            );
                        } else {
                            result_euc.extend_from_slice(
                                &format!(
                                    "&#x{:>02x}{:>02x}{:>02x}",
                                    utf8_buffer[utf8_i],
                                    utf8_buffer[utf8_i + 1],
                                    utf8_buffer[utf8_i + 2]
                                )
                                .as_bytes(),
                            );
                            if is_error_exit {
                                return Err(SkkError::Encoding);
                            }
                        }
                        utf8_i += 3;
                    }
                }
                0..=0x7f => {
                    result_euc.push(utf8_buffer[utf8_i]);
                    utf8_i += 1;
                }
                0xc2..=0xdf
                    if Encoder::is_utf8_2_bytes(&utf8_buffer, utf8_buffer_length, utf8_i) =>
                {
                    if Utility::is_enough_4_bytes(utf8_buffer_length, utf8_i)
                        && Encoder::is_candidate_combine_utf8_4(utf8_buffer, utf8_i)
                        && combine_utf8_4_to_euc_map.contains_key(&utf8_buffer[utf8_i..utf8_i + 4])
                    {
                        Encoder::push_to_buffer_euc(
                            &combine_utf8_4_to_euc_map[&utf8_buffer[utf8_i..utf8_i + 4]],
                            &mut result_euc,
                        );
                        utf8_i += 2 + 2;
                    } else {
                        let key = [utf8_buffer[utf8_i], utf8_buffer[utf8_i + 1], 0, 0];
                        if utf8_2_4_to_euc_map.contains_key(&key) {
                            Encoder::push_to_buffer_euc(
                                &utf8_2_4_to_euc_map[&key],
                                &mut result_euc,
                            );
                        } else {
                            result_euc.extend_from_slice(
                                &format!("&#x{:>02x}{:>02x}", key[0], key[1]).as_bytes(),
                            );
                            if is_error_exit {
                                return Err(SkkError::Encoding);
                            }
                        }
                        utf8_i += 2;
                    }
                }
                0xf0..=0xf7
                    if Encoder::is_utf8_4_bytes(&utf8_buffer, utf8_buffer_length, utf8_i) =>
                {
                    let key = &utf8_buffer[utf8_i..utf8_i + 4];
                    if utf8_2_4_to_euc_map.contains_key(key) {
                        Encoder::push_to_buffer_euc(&utf8_2_4_to_euc_map[key], &mut result_euc);
                    } else {
                        result_euc.extend_from_slice(
                            &format!(
                                "&#x{:>02x}{:>02x}{:>02x}{:>02x}",
                                key[0], key[1], key[2], key[3]
                            )
                            .as_bytes(),
                        );
                        if is_error_exit {
                            return Err(SkkError::Encoding);
                        }
                    }
                    utf8_i += 4;
                }
                _ => {
                    result_euc
                        .extend_from_slice(&format!("&#x{:>02x}", utf8_buffer[utf8_i]).as_bytes());
                    if is_error_exit {
                        return Err(SkkError::Encoding);
                    }
                    utf8_i += 1;
                }
            }
            assert!(utf8_i <= utf8_buffer_length);
            if utf8_i >= utf8_buffer_length {
                break;
            }
        }
        Ok(result_euc)
    }
}
