pub mod cplus;
pub trait FromBytes {
    type Err;

    fn from_bytes(s: &[u8]) -> Result<Self, Self::Err>
        where Self: Sized;
}

/// Test values taken directly from the protocol PDF
#[cfg(test)]
mod tests {
    use crate::model::FromBytes;
    use super::*;
    use std::{time::Duration};

    #[test]
    fn status_inquiry_test() {
        let cmd_response = b"208.4 140.0 208.4 034 59.9 2.05 35.0 00110000";
        let status = cplus::StatusInquiryResponse::from_bytes(cmd_response).unwrap();

        assert_eq!(status.temperature, 35.0);
        assert_eq!(status.battery_capacity, 62);
        assert!(status.ups_status.battery_abnormal);
    }

    #[test]
    fn ups_status_test() {
        let ups_status_string = b"00110000";
        let ups = cplus::UPSStatus::from_bytes(ups_status_string).unwrap();

        assert!(!ups.shutdown_active);
        assert!(ups.bypass_or_transformer_active);
        assert!(ups.battery_abnormal);
        assert!(!ups.offline);
    }

    #[test]
    fn extra_power_info_test() {
        let res = &[
            1, 244,
            0, 0,
            0, 0,
            5, 110,
            3, 182,
            2, 21,
            0, 0,
            0, 33,
            0, 0,
            0, 0,
        ];

        let xpi = cplus::ExtraPowerInfoResponse::from_bytes(res).unwrap();

        assert!(xpi.ups_output_freq == 50.0);
        assert!(xpi.battery_voltage == 13.9);
        assert!(xpi.battery_cut_voltage == 9.5);
        assert!(xpi.ups_wattage == 533);
        assert!(xpi.load_current == 3.3);
        // Not testing for the error as there is no documentation provided for it.
    }

    #[test]
    fn running_time_test() {
        let res = &[
            0, 0, // hi word
            5, 68 // 5 << 8 + 68 == 1348
        ];

        let autonomy = cplus::AutonomyResponse::from_bytes(res).unwrap();

        assert_eq!(autonomy.time, Duration::from_secs(1348))
    }

    #[test]
    fn running_time_test2() {
        let res = &[0x00, 0x01, 0x01, 0x01];

        let autonomy = cplus::AutonomyResponse::from_bytes(res).unwrap();

        assert_eq!(autonomy.time, Duration::from_secs(65793))
    }

    #[test]
    fn battery_life_test() {
        let res = &[
            0, 1,  // 1 << 16
            86, 48 //   + 86<<8 + 48  == 87600
        ];

        let life = cplus::BatteryLifeResponse::from_bytes(res).unwrap();

        assert_eq!(life.time.as_secs(), 60 * 60 * 87600);
    }

    #[test]
    fn ups_rating_test() {
        let res = b"230.0 008 072.0 50.0";
        
        let rating = cplus::UPSRating::from_bytes(res).unwrap();

        assert_eq!(rating.output_rating_voltage, 230.0);
        assert_eq!(rating.output_rating_current, 8);
        assert_eq!(rating.battery_voltage, 72.0);
        assert_eq!(rating.output_rating_frequency, 50.0);
    }
}
