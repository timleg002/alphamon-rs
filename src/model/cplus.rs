use serde::Serialize;
use tokio::time;
use crate::error;
use crate::{Result, error::Error};

use crate::model::FromBytes;

pub(crate) const BAUD_RATE: u32 = 2_400;

pub(crate) static CMD_STATUS_INQUIRY: &[u8] = b"Q1";
pub(crate) static CMD_ALARM_INQUIRY: &[u8] = b"Q4";
pub(crate) static CMD_EXTRA_POWER_PARAMETERS_INFO: &[u8] = b"Q5";

// Queries the UPS for the time it can run without AC power
pub(crate) static CMD_AUTONOMY: &[u8] = b"At";

// Queries the UPS for the estimated battery life
pub(crate) static CMD_BATTERY_LIFE: &[u8] = b"BL";

// Queries the UPS for its information
pub(crate) static CMD_UPS_INFORMATION: &[u8] = b"I";

// Queries the UPS for its information
pub(crate) static CMD_RATING_INFORMATION: &[u8] = b"F";

#[derive(Debug, Serialize)]
pub struct StatusInquiryResponse {
    /// V
    pub input_voltage: f32,
    /// "Do not use" (V)
    pub input_fault_voltage: f32, 
    /// V
    pub output_voltage: f32,
    /// %
    pub output_load_percentage: u32,
    /// Hz
    pub input_frequency: f32,
    /// %
    pub battery_capacity: u32,
    /// N/A
    pub battery_capacity_parameter: String,
    /// °C
    pub temperature: f32,
    pub ups_status: UPSStatus
}

impl FromBytes for StatusInquiryResponse {
    type Err = error::Error;

    fn from_bytes(s: &[u8]) -> Result<Self> {
        let [
            input_voltage,
            input_fault_voltage, 
            output_voltage,
            output_load_percentage,
            input_frequency,
            battery_capacity_parameter,
            temperature,
            ups_status
        ] = &s
            .split(|b| *b == b' ')
            .map(|x| String::from_utf8_lossy(x))
            .collect::<Vec<_>>()[..] else { return Err(Error::InvalidFormat) };

        let ups_status = UPSStatus::from_bytes(ups_status.as_bytes())?;

        let battery_capacity = match ups_status.offline {
            true => match battery_capacity_parameter as &str {
                "13.5" => 100,
                "13.3" => 90,
                "13.2" => 88,
                "13.1" => 86,
                "13" => 83,
                "12.9" => 80,
                "12.8" => 77,
                "12.7" => 74,
                "12.6" => 72,
                "12.5" => 69,
                "12.4" => 66,
                "12.3" => 63,
                "12.2" => 61,
                "12.1" => 58,
                "12" => 55,
                "11.9" => 52,
                "11.8" => 49,
                "11.7" => 47,
                "11.6" => 44,
                "11.5" => 41,
                "11.4" => 38,
                "11.3" => 36,
                "11.2" => 33,
                "11.1" => 30,
                "11" => 27,
                "10.9" => 24,
                "10.8" => 22,
                "10.7" => 19,
                "10.6" => 16,
                "10.5" => 13,
                "10.4" => 11,
                "10.3" => 8,
                "10.2" => 5,
                "10.1" => 2,
                "10" => 0,
                _ => return Err(Error::InvalidBatteryCapacityParameter),
            },
            false => match battery_capacity_parameter as &str {
                "2.22" => 100,
                "2.21" => 90,
                "2.20" => 88,
                "2.19" => 87,
                "2.18" => 85,
                "2.17" => 83,
                "2.16" => 82,
                "2.15" => 80,
                "2.14" => 78,
                "2.13" => 77,
                "2.12" => 75,
                "2.11" => 73,
                "2.10" => 72,
                "2.09" => 70,
                "2.08" => 68,
                "2.07" => 65,
                "2.06" => 65,
                "2.05" => 62,
                "2.04" => 62,
                "2.03" => 58,
                "2.02" => 58,
                "2.01" => 55,
                "2.00" => 55,
                "1.99" => 53,
                "1.98" => 52,
                "1.97" => 50,
                "1.96" => 48,
                "1.95" => 47,
                "1.94" => 45,
                "1.93" => 43,
                "1.92" => 42,
                "1.91" => 40,
                "1.90" => 38,
                "1.89" => 37,
                "1.88" => 35,
                "1.87" => 33,
                "1.86" => 32,
                "1.85" => 30,
                "1.84" => 28,
                "1.83" => 27,
                "1.82" => 25,
                "1.81" => 23,
                "1.80" => 22,
                "1.79" => 20,
                "1.78" => 18,
                "1.77" => 17,
                "1.76" => 15,
                "1.75" => 13,
                "1.74" => 12,
                "1.73" => 10,
                "1.72" => 8,
                "1.71" => 7,
                "1.70" => 5,
                "1.69" => 3,
                "1.68" => 2,
                "1.67" => 0,
                _ => return Err(Error::InvalidBatteryCapacityParameter),
            }    
        };

        Ok(Self {
            input_voltage: input_voltage.parse()?,
            input_fault_voltage: input_fault_voltage.parse()?,
            output_voltage: output_voltage.parse()?,
            output_load_percentage: output_load_percentage.parse()?,
            input_frequency: input_frequency.parse()?,
            battery_capacity_parameter: battery_capacity_parameter.to_string(),
            battery_capacity,
            temperature: temperature.parse()?,
            ups_status
        })
    }
}

#[derive(Debug, Serialize)]
pub struct UPSStatus {
    pub utility_fail: bool,
    pub battery_low: bool,
    /// If the UPS is in offline mode, the value signifies if the boost/buck converter is active,
    /// if the UPS is in online mode, it signifies if UPS is in bypass mode.
    pub bypass_or_transformer_active: bool,
    pub battery_abnormal: bool,
    pub offline: bool,
    pub test_in_progress: bool,
    pub shutdown_active: bool,
    pub beeper_on: bool
}

impl FromBytes for UPSStatus {
    type Err = error::Error;

    fn from_bytes(s: &[u8]) -> Result<Self> {
        let [
            utility_fail,
            battery_low,
            bypass_or_transformer_active,
            battery_abnormal,
            offline,
            test_in_progress,
            shutdown_active,
            beeper_on
        ] = s
            .iter()
            .map(|x| *x == b'1')
            .collect::<Vec<bool>>()[..] else { return Err(Error::InvalidFormat) };

        Ok(Self {
            utility_fail,
            battery_low,
            bypass_or_transformer_active,
            battery_abnormal,
            offline,
            test_in_progress,
            shutdown_active,
            beeper_on,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AlarmInquiryResponse {
    pub inverter_on: bool,
    pub ups_alarm_on: bool
}

impl FromBytes for AlarmInquiryResponse {
    type Err = error::Error;

    fn from_bytes(s: &[u8]) -> Result<Self> {
        let [
            inverter_on,
            ups_alarm_on
        ] = s
            .get(0..2)
            .ok_or(Error::InvalidFormat)?
            .iter()
            .map(|x| *x == b'1')
            .collect::<Vec<bool>>()[..] else { return Err(Error::InvalidFormat) };

        Ok(Self {
            inverter_on,
            ups_alarm_on
        })
    }
}
    
#[derive(Debug, Serialize)]
pub struct ExtraPowerInfoResponse {
    /// Hz
    pub ups_output_freq: f32,
    /// V
    pub battery_voltage: f32,
    /// V
    pub battery_cut_voltage: f32,
    /// W
    pub ups_wattage: u32,
    /// Undocumented error code.
    pub error_code: u16,
    /// A
    pub load_current: f32
}

impl FromBytes for ExtraPowerInfoResponse {
    type Err = error::Error;

    fn from_bytes(s: &[u8]) -> Result<Self> {
        let [
            fout, 
            _,
            _,
            vb,
            vbc,
            inv_w,
            ercode,
            o_cur,
            _,
            _,
        ] = &s
            .chunks(2)
            .map(|s| s.try_into().map(u16::from_be_bytes).map(f32::from).map_err(|e| e.into()))
            .collect::<Result<Vec<f32>>>()?[..] else { return Err(Error::InvalidFormat) };

        Ok(Self {
            ups_output_freq: fout * 0.1,
            battery_voltage: vb * 0.01,
            battery_cut_voltage: vbc * 0.01,
            ups_wattage: *inv_w as u32,
            error_code: *ercode as u16,
            load_current: o_cur * 0.1,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AutonomyResponse {
    pub time: time::Duration,
}

impl FromBytes for AutonomyResponse {
    type Err = error::Error;

    fn from_bytes(s: &[u8]) -> Result<Self> {    
        let time = time::Duration::from_secs(
            u32::from_be_bytes(s.try_into()?) as u64
        );

        Ok(Self {
            time
        })
    }
}

#[derive(Debug, Serialize)]
pub struct BatteryLifeResponse {
    pub time: time::Duration,
}

impl FromBytes for BatteryLifeResponse {
    type Err = error::Error;

    fn from_bytes(s: &[u8]) -> Result<Self> {
        let time = time::Duration::from_secs(
            u32::from_be_bytes(s.try_into()?) as u64 * 60 * 60
        );

        Ok(Self {
            time
        })
    }
}

#[derive(Debug, Serialize)]
pub struct UPSInformation {
    pub manufacturer_name: String,
    pub model: String,
    pub version: String
}

impl FromBytes for UPSInformation {
    type Err = error::Error;

    fn from_bytes(s: &[u8]) -> Result<Self> {
        let (mfg_name, s) = s.split_at(15);
        let (model, s) = s.split_at(10);
        let (version, _) = s.split_at(10);

        Ok(Self {
            manufacturer_name: String::from_utf8_lossy(mfg_name).trim().to_string(),
            model: String::from_utf8_lossy(model).trim().to_string(),
            version: String::from_utf8_lossy(version).trim().to_string()
        })
    }
}

#[derive(Debug, Serialize)]
pub struct UPSRating {
    /// V
    pub output_rating_voltage: f32,
    /// A
    pub output_rating_current: u32,
    /// V
    pub battery_voltage: f32,
    /// Hz
    pub output_rating_frequency: f32
}

impl FromBytes for UPSRating {
    type Err = error::Error;

    fn from_bytes(s: &[u8]) -> Result<Self> {
        let [
            output_rating_voltage,
            output_rating_current,
            battery_voltage,
            output_rating_frequency
        ] = &s
            .split(|byte| *byte == b' ')
            .map(|bytes| String::from_utf8_lossy(bytes))
            .collect::<Vec<_>>()[..] else { return Err(Error::InvalidFormat) };

        Ok(Self {
            output_rating_voltage: output_rating_voltage.parse()?,
            output_rating_current: output_rating_current.parse()?,
            battery_voltage: battery_voltage.parse()?,
            output_rating_frequency: output_rating_frequency.parse()?
        })
    }
}