/*
 * Copyright 2008 ZXing authors
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

use std::fmt;

use crate::qrcode::decoder::{Mode, ErrorCorrectionLevel, Version};

use super::ByteMatrix;

 const NUM_MASK_PATTERNS : i32 = 8;

/**
 * @author satorux@google.com (Satoru Takabayashi) - creator
 * @author dswitkin@google.com (Daniel Switkin) - ported from C++
 */
pub struct QRCode {

  // public static final int NUM_MASK_PATTERNS = 8;

    mode:Option<Mode>,
    ecLevel:Option<ErrorCorrectionLevel>,
    version:Option<&'static Version>,
   maskPattern:i32,
    matrix:Option<ByteMatrix>,

}

impl QRCode {
  pub fn new()->Self {
    Self {
        mode: None,
        ecLevel: None,
        version: None,
        maskPattern: -1,
        matrix: None,
    }
  }

  /**
   * @return the mode. Not relevant if {@link com.google.zxing.EncodeHintType#QR_COMPACT} is selected.
   */
  pub fn getMode(&self) -> &Option<Mode>{
    &self.mode
  }

   pub fn getECLevel(&self) -> &Option<ErrorCorrectionLevel>{
    &self.ecLevel
  }

  pub fn getVersion(&self) -> &Option<&'static Version>{
    &self.version
  }

  pub fn getMaskPattern(&self) -> i32{
    self.maskPattern
  }

  pub fn getMatrix(&self) ->&Option<ByteMatrix>{
    &self.matrix
  }

  

  pub fn setMode(&mut self, value:Mode) {
    self.mode = Some(value);
  }

  pub fn setECLevel(&mut self,  value:ErrorCorrectionLevel) {
    self.ecLevel = Some(value);
  }

  pub fn setVersion(&mut self,  version:&'static Version) {
    self.version = Some(version);
  }

  pub fn setMaskPattern(&mut self,  value:i32) {
    self.maskPattern = value;
  }

  pub fn setMatrix(&mut self, value:ByteMatrix) {
    self.matrix = Some(value);
  }

  // Check if "mask_pattern" is valid.
  pub fn isValidMaskPattern(  maskPattern:i32)  -> bool{
     maskPattern >= 0 && maskPattern < NUM_MASK_PATTERNS
  }

}

impl fmt::Display for QRCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let mut result =  String::with_capacity(200);
      result.push_str("<<\n");
      result.push_str(" mode: ");
      result.push_str(&format!("{:?}", self.mode));
      result.push_str("\n ecLevel: ");
      result.push_str(&format!("{:?}", self.ecLevel));
      result.push_str("\n version: ");
      result.push_str(&format!("{}",self.version.as_ref().unwrap()));
      result.push_str("\n maskPattern: ");
      result.push_str(&format!("{}",self.maskPattern));
      if self.matrix.is_none() {
        result.push_str("\n matrix: null\n");
      } else {
        result.push_str("\n matrix:\n");
        result.push_str(&format!("{}",self.matrix.as_ref().unwrap()));
      }
      result.push_str(">>\n");
      
      write!(f, "{}", result)
    }
}