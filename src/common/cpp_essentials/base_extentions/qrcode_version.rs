use crate::common::Result;
use crate::qrcode::decoder::{Version, VersionRef, VERSION_DECODE_INFO};
use crate::Exceptions;

impl Version {
    pub fn DimensionOfVersion(version: u32, is_micro: bool) -> u32 {
        Self::DimensionOffset(is_micro) + Self::DimensionStep(is_micro) * version
    }

    pub fn DimensionOffset(is_micro: bool) -> u32 {
        match is_micro {
            true => 9,
            false => 17,
        }
        // return std::array{17, 9}[isMicro];
    }

    pub fn DimensionStep(is_micro: bool) -> u32 {
        match is_micro {
            true => 2,
            false => 4,
        }
        // return std::array{4, 2}[isMicro];
    }
    pub fn DecodeVersionInformation(versionBitsA: i32, versionBitsB: i32) -> Result<VersionRef> {
        let mut bestDifference = u32::MAX;
        let mut bestVersion = 0;
        let mut i = 0;
        for targetVersion in VERSION_DECODE_INFO {
            // for (int targetVersion : VERSION_DECODE_INFO) {
            // Do the version info bits match exactly? done.
            if targetVersion == versionBitsA as u32 || targetVersion == versionBitsB as u32 {
                return Self::getVersionForNumber(i + 7);
            }
            // Otherwise see if this is the closest to a real version info bit string
            // we have seen so far
            for bits in [versionBitsA, versionBitsB] {
                // for (int bits : {versionBitsA, versionBitsB}) {
                let bitsDifference = ((bits as u32) ^ targetVersion).count_ones(); //BitHacks::CountBitsSet(bits ^ targetVersion);
                if bitsDifference < bestDifference {
                    bestVersion = i + 7;
                    bestDifference = bitsDifference;
                }
            }
            i += 1;
        }
        // We can tolerate up to 3 bits of error since no two version info codewords will
        // differ in less than 8 bits.
        if bestDifference <= 3 {
            return Self::getVersionForNumber(bestVersion);
        }
        // If we didn't find a close enough match, fail
        return Err(Exceptions::ILLEGAL_STATE);
    }
}
