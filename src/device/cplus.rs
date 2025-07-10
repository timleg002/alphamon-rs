use crate::Result;
use crate::error;
use crate::model::FromBytes;
use crate::model::cplus;
use std::ffi::CString;
use std::io::Write;
use std::time::Duration;

/// End byte of CPlus messages.
const END_BYTE: u8 = b'\r';

/// This USB HID feature report continuosly sends a carousel of messages
const DATA_FEATURE_REPORT: u8 = 5;

/// Prefix of the UPSStatus message. Also matches the prefix for other messages,
/// such as UPSExtraInfo.
const STATUS_MSG_PREFIX: u8 = b'(';
/// Prefix of the UPSRating message.
const RATING_MSG_PREFIX: u8 = b'#';

pub trait CPlusInterface {
    /// Queries the input/output voltage, load percentage, input AC frequency,
    /// battery capacity temperature and the UPS status and errors/warnings (battery, etc.)
    fn query_ups_status(&mut self) -> Result<cplus::StatusInquiryResponse>;

    /// Queries the UPS output AC frequency, per-battery voltage, UPS load in watts,
    /// UPS error code, and the UPS load current in amperes.
     fn query_extra_power_info(&mut self) -> Result<cplus::ExtraPowerInfoResponse>;

    /// Queries the UPS for alarm notifications: whether the inverter is on or off,
    /// or if the UPS itself is in the state of an alarm.
     fn query_alarm(&mut self) -> Result<cplus::AlarmInquiryResponse>;

    /// Queries the UPS for the length of time during which the UPS can
    /// provide power, taking in account the current load and battery capacity.
     fn query_ups_autonomy(&mut self) -> Result<cplus::AutonomyResponse>;

    /// Queries the UPS for the remaining lifetime of its battery.
     fn query_ups_battery_life(&mut self) -> Result<cplus::BatteryLifeResponse>;

    /// Queries the UPS for info about its manufacturer, model name and version.
     fn query_ups_info(&mut self) -> Result<cplus::UPSInformation>;

    /// Queries the UPS for info about its rated output voltage, current, frequency and battery voltage.
     fn query_ups_rating(&mut self) -> Result<cplus::UPSRating>;
}

#[cfg(feature = "serial")]
#[derive(Debug)]
pub struct CPlusSerialInterface {
    port: Box<dyn serialport::SerialPort>,
}

#[cfg(feature = "serial")]
impl CPlusSerialInterface {
    /// Connects to the serial port at the provided path with a 5s timeout.
    pub fn connect(port_path: &str) -> Result<Self> {
        let mut port = serialport::new(port_path, cplus::SERIAL_BAUD_RATE)
            .timeout(Duration::from_millis(5000))
            .open()?;

        port.write_data_terminal_ready(true)?;

        Ok(CPlusSerialInterface { port })
    }

    /// Writes data to the serial port along with the end byte.
     fn write_data(&mut self, msg: &[u8]) -> Result<()> {
        self.port.write_all(msg)?;
        self.port.write_all(&[END_BYTE])?;

        trace!("Wrote msg {:?}", String::from_utf8_lossy(msg));

        Ok(())
    }

    /// Reads data from the serial port until an end byte (CR) is encountered.
     fn read_data(&mut self) -> Result<Vec<u8>> {
        let mut buf = vec![];

        trace!("Reading buffer");

        let mut byte = [0u8];

        // Inefficient. Though it doesn't matter for small amounts of data.
        // In addition, the `read_exact` function handles some benign I/O errors in itself.
        while self.port.read_exact(&mut byte).is_ok() {
            if byte[0] == END_BYTE {
                break;
            }

            buf.push(byte[0]);
        }

        trace!("Read buffer {:?}\n", String::from_utf8_lossy(&buf));

        Ok(buf)
    }

    /// Queries - writes a command and awaits its response
     fn raw_query(&mut self, query: &[u8]) -> Result<Vec<u8>> {
        trace!("Querying with message {:?}", String::from_utf8_lossy(query));

        // A synchronization error can cause a partial packet to be in the input buffer
        self.port.clear(serialport::ClearBuffer::All)?;

        self.write_data(query)?;
        let output = self.read_data()?;

        Ok(output)
    }

    /// Queries the device and returns the processed response as a struct
    fn processed_query<T>(&mut self, query: &[u8]) -> Result<T>
    where
        T: FromBytes,
        <T as FromBytes>::Err: Into<error::Error>,
    {
        let raw_query = self.raw_query(query)?;

        // Remove the start byte
        let Some(processed_bytes) = &raw_query.get(1..) else {
            return Err(error::Error::InvalidFormat);
        };

        T::from_bytes(processed_bytes).map_err(|e| e.into())
    }
}

#[cfg(feature = "serial")]
impl CPlusInterface for CPlusSerialInterface {
    /// Queries the input/output voltage, load percentage, input AC frequency,
    /// battery capacity temperature and the UPS status and errors/warnings (battery, etc.)
     fn query_ups_status(&mut self) -> Result<cplus::StatusInquiryResponse> {
        self.processed_query(cplus::CMD_STATUS_INQUIRY)
    }

    /// Queries the UPS output AC frequency, per-battery voltage, UPS load in watts,
    /// UPS error code, and the UPS load current in amperes.
     fn query_extra_power_info(&mut self) -> Result<cplus::ExtraPowerInfoResponse> {
        self.processed_query(cplus::CMD_EXTRA_POWER_PARAMETERS_INFO)
            
    }

    /// Queries the UPS for alarm notifications: whether the inverter is on or off,
    /// or if the UPS itself is in the state of an alarm.
     fn query_alarm(&mut self) -> Result<cplus::AlarmInquiryResponse> {
        self.processed_query(cplus::CMD_ALARM_INQUIRY)
    }

    /// Queries the UPS for the length of time during which the UPS can
    /// provide power, taking in account the current load and battery capacity.
     fn query_ups_autonomy(&mut self) -> Result<cplus::AutonomyResponse> {
        self.processed_query(cplus::CMD_AUTONOMY)
    }

    /// Queries the UPS for the remaining lifetime of its battery.
     fn query_ups_battery_life(&mut self) -> Result<cplus::BatteryLifeResponse> {
        self.processed_query(cplus::CMD_BATTERY_LIFE)
    }

    /// Queries the UPS for info about its manufacturer, model name and version.
     fn query_ups_info(&mut self) -> Result<cplus::UPSInformation> {
        self.processed_query(cplus::CMD_UPS_INFORMATION)
    }

    /// Queries the UPS for info about its rated output voltage, current, frequency and battery voltage.
     fn query_ups_rating(&mut self) -> Result<cplus::UPSRating> {
        self.processed_query(cplus::CMD_RATING_INFORMATION)
    }
}

#[cfg(feature = "usb-hidapi")]
pub struct CPlusHidInterface {
    api: hidapi::HidApi,
    device: hidapi::HidDevice,
}

#[cfg(feature = "usb-hidapi")]
impl CPlusHidInterface {
    /// Connects to the given HID device at `path`.
    pub fn connect_with_path(path: String) -> Result<Self> {
        let api = hidapi::HidApi::new()?;

        let path = path.replace("\0", "");

        // All null bytes were removed by the previous call.
        let path = CString::new(path).unwrap();

        let device = api.open_path(path.as_c_str())?;

        Ok(Self { api, device })
    }

    /// Connects to the given HID device with the given `vid` and `pid`.
    pub fn connect_with_vid_pid(vid: u16, pid: u16) -> Result<Self> {
        let api = hidapi::HidApi::new()?;

        let device = api.open(vid, pid)?;

        Ok(Self { api, device })
    }

    /// Reads raw data from the data feature report.
    /// The buffer is expected to be at least 2 bytes long.
    fn read_raw_data(&mut self, buf: &mut [u8]) -> Result<usize> {
        let report_id = buf.get_mut(0).ok_or(error::Error::BufferTooSmall {
            expected: 1,
            provided: 0,
        })?;

        *report_id = DATA_FEATURE_REPORT;

        // The doc of this function says that,
        // "Upon return, the first byte will still contain the Report ID,
        // and the report data will start in buf[1]."
        // Which doesn't apply for this UPS, the data starts in buf[0]
        let read = self.device.get_feature_report(buf)?;

        Ok(read)
    }

    /// Reads data from the feature report. If a `start_idx` is provided,
    /// the function will read until a complete message with the given `start_idx` is found.
    /// The `start_idx` represents the type of message received.
    /// 
    /// Returns the position of the end byte (a carriage return character).
    fn read_data(&mut self, buf: &mut [u8], start_idx: Option<u8>) -> Result<usize> {
        let cr_idx = loop {
            self.read_raw_data(buf)?;

            let Some(cr_idx) = buf.iter().position(|&b| b == END_BYTE) else {
                continue;
            };

            if (cr_idx + 1 == buf.len() || *buf.get(cr_idx + 1).unwrap() == b'\0') 
                && (start_idx.is_none() || buf.first() == start_idx.as_ref())
            {
                break cr_idx
            }
        };

        Ok(cr_idx)
    }

    fn read_processed_data<T>(&mut self, start_idx: Option<u8>) -> Result<T> 
        where T: FromBytes, <T as FromBytes>::Err: Into<error::Error> 
    {
        let mut buf = vec![0u8; 48];

        let cr_idx = self.read_data(&mut buf, start_idx)?;

        // First byte is the start_idx, the message ends with the end byte (carriage return)
        let Some(processed_bytes) = &buf.get(1..cr_idx) else {
            return Err(error::Error::InvalidFormat);
        };

        T::from_bytes(processed_bytes).map_err(|e| e.into())
    }
}

#[cfg(feature = "usb-hidapi")]
impl CPlusInterface for CPlusHidInterface {
     fn query_ups_status(&mut self) -> Result<cplus::StatusInquiryResponse> {
        self.read_processed_data(Some(STATUS_MSG_PREFIX))
    }
    
     fn query_ups_rating(&mut self) -> Result<cplus::UPSRating> {
        self.read_processed_data(Some(RATING_MSG_PREFIX))
    }

     fn query_extra_power_info(&mut self) -> Result<cplus::ExtraPowerInfoResponse> {
        unimplemented!("HID interface USB v0.2 only supports status and rating queries")
    }

     fn query_alarm(&mut self) -> Result<cplus::AlarmInquiryResponse> {
        unimplemented!("HID interface USB v0.2 only supports status and rating queries")
    }

     fn query_ups_autonomy(&mut self) -> Result<cplus::AutonomyResponse> {
        unimplemented!("HID interface USB v0.2 only supports status and rating queries")
    }

     fn query_ups_battery_life(&mut self) -> Result<cplus::BatteryLifeResponse> {
        unimplemented!("HID interface USB v0.2 only supports status and rating queries")
    }

     fn query_ups_info(&mut self) -> Result<cplus::UPSInformation> {
        unimplemented!("HID interface USB v0.2 only supports status and rating queries")
    }
}