/*
 * Copyright 2010 ZXing authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use one_d_reader_derive::OneDReader;

use crate::{common::BitArray, BarcodeFormat, Exceptions, RXingResult};

use super::OneDReader;

/**
 * <p>Decodes Code 93 barcodes.</p>
 *
 * @author Sean Owen
 * @see Code39Reader
 */
#[derive(OneDReader)]
pub struct Code93Reader {
    decodeRowRXingResult: String,
    counters: [u32; 6],
}

impl OneDReader for Code93Reader {
    fn decodeRow(
        &mut self,
        rowNumber: u32,
        row: &crate::common::BitArray,
        _hints: &crate::DecodingHintDictionary,
    ) -> Result<crate::RXingResult, Exceptions> {
        let start = self.findAsteriskPattern(row)?;
        // Read off white space
        let mut nextStart = row.getNextSet(start[1]);
        let end = row.getSize();

        let mut theCounters = self.counters;
        theCounters.fill(0);
        self.decodeRowRXingResult.truncate(0);

        let mut decodedChar;
        let mut lastStart;
        loop {
            self.recordPattern(row, nextStart, &mut theCounters)?;
            let pattern = Self::toPattern(&theCounters);
            if pattern < 0 {
                return Err(Exceptions::NotFoundException("".to_owned()));
            }
            decodedChar = Self::patternToChar(pattern as u32)?;
            self.decodeRowRXingResult.push(decodedChar);
            lastStart = nextStart;
            for counter in theCounters {
                // for (int counter : theCounters) {
                nextStart += counter as usize;
            }
            // Read off white space
            nextStart = row.getNextSet(nextStart);

            if !(decodedChar != '*') {
                break;
            }
        } //while (decodedChar != '*');
        self.decodeRowRXingResult
            .truncate(self.decodeRowRXingResult.chars().count() - 1); // remove asterisk

        // let mut lastPatternSize = 0;
        // for counter in theCounters {
        //     // for (int counter : theCounters) {
        //     lastPatternSize += counter;
        // }

        let lastPatternSize: u32 = theCounters.iter().sum();

        // Should be at least one more black module
        if nextStart == end || !row.get(nextStart) {
            return Err(Exceptions::NotFoundException("".to_owned()));
        }

        if self.decodeRowRXingResult.chars().count() < 2 {
            // false positive -- need at least 2 checksum digits
            return Err(Exceptions::NotFoundException("".to_owned()));
        }

        Self::checkChecksums(&self.decodeRowRXingResult)?;
        // Remove checksum digits
        self.decodeRowRXingResult
            .truncate(self.decodeRowRXingResult.chars().count() - 2);

        let resultString = Self::decodeExtended(&self.decodeRowRXingResult)?;

        let left: f32 = (start[1] + start[0]) as f32 / 2.0;
        let right: f32 = lastStart as f32 + lastPatternSize as f32 / 2.0;

        let mut resultObject = RXingResult::new(
            &resultString,
            Vec::new(),
            vec![
                RXingResultPoint::new(left, rowNumber as f32),
                RXingResultPoint::new(right, rowNumber as f32),
            ],
            BarcodeFormat::CODE_93,
        );

        resultObject.putMetadata(
            RXingResultMetadataType::SYMBOLOGY_IDENTIFIER,
            RXingResultMetadataValue::SymbologyIdentifier("]G0".to_owned()),
        );

        Ok(resultObject)
    }
}

impl Code93Reader {
    // Note that 'abcd' are dummy characters in place of control characters.
    const ALPHABET_STRING: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ-. $/+%abcd*";
    const ALPHABET: [char; 48] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
        'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        '-', '.', ' ', '$', '/', '+', '%', 'a', 'b', 'c', 'd', '*',
    ];

    /**
     * These represent the encodings of characters, as patterns of wide and narrow bars.
     * The 9 least-significant bits of each int correspond to the pattern of wide and narrow.
     */
    const CHARACTER_ENCODINGS: [u32; 48] = [
        0x114, 0x148, 0x144, 0x142, 0x128, 0x124, 0x122, 0x150, 0x112, 0x10A, // 0-9
        0x1A8, 0x1A4, 0x1A2, 0x194, 0x192, 0x18A, 0x168, 0x164, 0x162, 0x134, // A-J
        0x11A, 0x158, 0x14C, 0x146, 0x12C, 0x116, 0x1B4, 0x1B2, 0x1AC, 0x1A6, // K-T
        0x196, 0x19A, 0x16C, 0x166, 0x136, 0x13A, // U-Z
        0x12E, 0x1D4, 0x1D2, 0x1CA, 0x16E, 0x176, 0x1AE, // - - %
        0x126, 0x1DA, 0x1D6, 0x132, 0x15E, // Control chars? $-*
    ];
    const ASTERISK_ENCODING: i32 = Self::CHARACTER_ENCODINGS[47] as i32;

    pub fn new() -> Self {
        Self {
            decodeRowRXingResult: String::with_capacity(20),
            counters: [0; 6],
        }
    }

    fn findAsteriskPattern(&mut self, row: &BitArray) -> Result<[usize; 2], Exceptions> {
        let width = row.getSize();
        let rowOffset = row.getNextSet(0);

        self.counters.fill(0);
        let mut theCounters = self.counters;
        let mut patternStart = rowOffset;
        let mut isWhite = false;
        let patternLength = theCounters.len();

        let mut counterPosition = 0;
        for i in rowOffset..width {
            // for (int i = rowOffset; i < width; i++) {
            if row.get(i) != isWhite {
                theCounters[counterPosition] += 1;
            } else {
                if counterPosition == patternLength - 1 {
                    if Self::toPattern(&theCounters) == Self::ASTERISK_ENCODING {
                        return Ok([patternStart, i]);
                    }
                    patternStart += (theCounters[0] + theCounters[1]) as usize;
                    let slc = &theCounters[2..(counterPosition - 1 + 2)].to_vec();
                    theCounters[0..(counterPosition - 1)].copy_from_slice(slc);
                    // System.arraycopy(theCounters, 2, theCounters, 0, counterPosition - 1);
                    theCounters[counterPosition - 1] = 0;
                    theCounters[counterPosition] = 0;
                    counterPosition -= 1;
                } else {
                    counterPosition += 1;
                }
                theCounters[counterPosition] = 1;
                isWhite = !isWhite;
            }
        }
        return Err(Exceptions::NotFoundException("".to_owned()));
    }

    fn toPattern(counters: &[u32; 6]) -> i32 {
        let mut sum = 0;
        for counter in counters {
            // for (int counter : counters) {
            sum += counter;
        }
        let mut pattern = 0;
        let max = counters.len();
        for i in 0..max {
            // for (int i = 0; i < max; i++) {
            let scaled = (counters[i] as f32 * 9.0 / sum as f32).round() as u32;
            // let scaled = Math.round(counters[i] * 9.0 / sum);
            if scaled < 1 || scaled > 4 {
                return -1;
            }
            if (i & 0x01) == 0 {
                for _j in 0..scaled {
                    // for (int j = 0; j < scaled; j++) {
                    pattern = (pattern << 1) | 0x01;
                }
            } else {
                pattern <<= scaled;
            }
        }
        return pattern;
    }

    fn patternToChar(pattern: u32) -> Result<char, Exceptions> {
        for i in 0..Self::CHARACTER_ENCODINGS.len() {
            // for (int i = 0; i < CHARACTER_ENCODINGS.length; i++) {
            if Self::CHARACTER_ENCODINGS[i] == pattern {
                return Ok(Self::ALPHABET[i]);
            }
        }
        return Err(Exceptions::NotFoundException("".to_owned()));
    }

    fn decodeExtended(encoded: &str) -> Result<String, Exceptions> {
        let length = encoded.chars().count();
        let mut decoded = String::with_capacity(length);
        let mut i = 0;
        while i < length {
            // for i in 0..length {
            // for (int i = 0; i < length; i++) {
            let c = encoded.chars().nth(i).unwrap();
            if c >= 'a' && c <= 'd' {
                if i >= length - 1 {
                    return Err(Exceptions::FormatException("".to_owned()));
                }
                let next = encoded.chars().nth(i + 1).unwrap();
                let mut decodedChar = '\0';
                match c {
                    'd' => {
                        // +A to +Z map to a to z
                        if next >= 'A' && next <= 'Z' {
                            decodedChar = char::from_u32(next as u32 + 32).unwrap();
                        } else {
                            return Err(Exceptions::FormatException("".to_owned()));
                        }
                    }
                    'a' => {
                        // $A to $Z map to control codes SH to SB
                        if next >= 'A' && next <= 'Z' {
                            decodedChar = char::from_u32(next as u32 - 64).unwrap();
                        } else {
                            return Err(Exceptions::FormatException("".to_owned()));
                        }
                    }
                    'b' => {
                        if next >= 'A' && next <= 'E' {
                            // %A to %E map to control codes ESC to USep
                            decodedChar = char::from_u32(next as u32 - 38).unwrap();
                        } else if next >= 'F' && next <= 'J' {
                            // %F to %J map to ; < = > ?
                            decodedChar = char::from_u32(next as u32 - 11).unwrap();
                        } else if next >= 'K' && next <= 'O' {
                            // %K to %O map to [ \ ] ^ _
                            decodedChar = char::from_u32(next as u32 + 16).unwrap();
                        } else if next >= 'P' && next <= 'T' {
                            // %P to %T map to { | } ~ DEL
                            decodedChar = char::from_u32(next as u32 + 43).unwrap();
                        } else if next == 'U' {
                            // %U map to NUL
                            decodedChar = '\0';
                        } else if next == 'V' {
                            // %V map to @
                            decodedChar = '@';
                        } else if next == 'W' {
                            // %W map to `
                            decodedChar = '`';
                        } else if next >= 'X' && next <= 'Z' {
                            // %X to %Z all map to DEL (127)
                            decodedChar = 127 as char;
                        } else {
                            return Err(Exceptions::FormatException("".to_owned()));
                        }
                    }
                    'c' => {
                        // /A to /O map to ! to , and /Z maps to :
                        if next >= 'A' && next <= 'O' {
                            decodedChar = char::from_u32(next as u32 - 32).unwrap();
                        } else if next == 'Z' {
                            decodedChar = ':';
                        } else {
                            return Err(Exceptions::FormatException("".to_owned()));
                        }
                    }
                    _ => {}
                }
                decoded.push(decodedChar);
                // bump up i again since we read two characters
                i += 1;
            } else {
                decoded.push(c);
            }

            i += 1;
        }
        Ok(decoded)
    }

    fn checkChecksums(result: &str) -> Result<(), Exceptions> {
        let length = result.chars().count();
        Self::checkOneChecksum(result, length - 2, 20)?;
        Self::checkOneChecksum(result, length - 1, 15)?;
        Ok(())
    }

    fn checkOneChecksum(
        result: &str,
        checkPosition: usize,
        weightMax: u32,
    ) -> Result<(), Exceptions> {
        let mut weight = 1;
        let mut total = 0;
        for i in (0..checkPosition).rev() {
            // for (int i = checkPosition - 1; i >= 0; i--) {
            total += weight
                * if let Some(pos) = Self::ALPHABET_STRING.find(result.chars().nth(i).unwrap()) {
                    pos as i32
                } else {
                    -1
                };
            weight += 1;
            if weight > weightMax as i32 {
                weight = 1;
            }
        }
        if result.chars().nth(checkPosition).unwrap() != Self::ALPHABET[(total as usize) % 47] {
            Err(Exceptions::ChecksumException("".to_owned()))
        } else {
            Ok(())
        }
    }
}

/**
 * @author Daisuke Makiuchi
 */
#[cfg(test)]
mod Code93ReaderTestCase {
    use std::collections::HashMap;

    use crate::{
        common::{BitArray, BitMatrix},
        oned::OneDReader,
    };

    use super::Code93Reader;

    #[test]
    fn testDecode() {
        doTest("Code93!\n$%/+ :\u{001b};[{\u{007f}\u{0000}@`\u{007f}\u{007f}\u{007f}",
             "0000001010111101101000101001100101001011001001100101100101001001100101100100101000010101010000101110101101101010001001001101001101001110010101101011101011011101011101101110100101110101101001110101110110101101010001110110101100010101110110101000110101110110101000101101110110101101001101110110101100101101110110101100110101110110101011011001110110101011001101110110101001101101110110101001110101001100101101010001010111101111");
    }

    fn doTest(expectedRXingResult: &str, encodedRXingResult: &str) {
        let mut sut = Code93Reader::new();
        let matrix = BitMatrix::parse_strings(encodedRXingResult, "1", "0").expect("must parse");
        let mut row = BitArray::with_size(matrix.getWidth() as usize);
        row = matrix.getRow(0, row);
        let result = sut
            .decodeRow(0, &row, &HashMap::new())
            .expect("must decode");
        assert_eq!(expectedRXingResult, result.getText());
    }
}