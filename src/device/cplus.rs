use std::io::{Write};
use std::time::Duration;
use crate::error;
use crate::Result;
use serialport::{ClearBuffer};
use crate::model::cplus::{self};
use crate::model::FromBytes;

const END_BYTE: u8 = b'\r'; 

#[derive(Debug)]
pub struct CPlusSerialInterface {
    port: Box<dyn serialport::SerialPort>,
}

impl CPlusSerialInterface {
    /// Connects to the serial port at the provided path with a 5s timeout.
    pub fn connect(port_path: &str) -> Result<Self> {
        let mut port = match serialport::new(port_path, cplus::BAUD_RATE)
            .timeout(Duration::from_millis(5000))
            .open() 
        {
            Ok(port) => port,
            Err(err) => {
                return Err(err.into())
            },
        };

        port.write_data_terminal_ready(true)?;

        Ok(CPlusSerialInterface {
            port
        })
    }

    /// Writes data to the serial port along with the end byte.
    async fn write_data(&mut self, msg: &[u8]) -> Result<()> {
        self.port.write_all(msg)?;
        self.port.write_all(&[END_BYTE])?;

        trace!("Wrote msg {:?}", String::from_utf8_lossy(msg));

        Ok(())
    }

    /// Reads data from the serial port until an end byte (CR) is encountered.
    async fn read_data(&mut self) -> Result<Vec<u8>> {
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
    async fn raw_query(&mut self, query: &[u8]) -> Result<Vec<u8>> {
        trace!("Querying with message {:?}", String::from_utf8_lossy(query));

        // A synchronization error can cause a partial packet to be in the input buffer
        self.port.clear(ClearBuffer::All)?;

        self.write_data(query).await?;
        let output = self.read_data().await?;

        Ok(output)
    }

    /// Queries the device and returns the processed response as a struct
    async fn processed_query<T>(&mut self, query: &[u8]) -> Result<T>
        where T: FromBytes, <T as FromBytes>::Err: Into<error::Error>
    {
        let raw_query = self.raw_query(query).await?;

        // Remove the start byte
        let Some(processed_bytes) = &raw_query.get(1..) else {
            return Err(error::Error::InvalidFormat)
        };

        T::from_bytes(processed_bytes).map_err(|e| e.into())
    }

    /// Queries the input/output voltage, load percentage, input AC frequency,
    /// battery capacity temperature and the UPS status and errors/warnings (battery, etc.)
    pub async fn query_ups_status(&mut self) -> Result<cplus::StatusInquiryResponse> {
        self.processed_query(cplus::CMD_STATUS_INQUIRY).await
    }

    /// Queries the UPS output AC frequency, per-battery voltage, UPS load in watts,
    /// UPS error code, and the UPS load current in amperes.
    pub async fn query_extra_power_info(&mut self) -> Result<cplus::ExtraPowerInfoResponse> {
        self.processed_query(cplus::CMD_EXTRA_POWER_PARAMETERS_INFO).await
    }

    /// Queries the UPS for alarm notifications: whether the inverter is on or off,
    /// or if the UPS itself is in the state of an alarm.
    pub async fn query_alarm(&mut self) -> Result<cplus::AlarmInquiryResponse> {
        self.processed_query(cplus::CMD_ALARM_INQUIRY).await
    }

    /// Queries the UPS for the length of time during which the UPS can
    /// provide power, taking in account the current load and battery capacity.
    pub async fn query_ups_autonomy(&mut self) -> Result<cplus::AutonomyResponse> {
        self.processed_query(cplus::CMD_AUTONOMY).await
    }

    /// Queries the UPS for the remaining lifetime of its battery.
    pub async fn query_ups_battery_life(&mut self) -> Result<cplus::BatteryLifeResponse> {
        self.processed_query(cplus::CMD_BATTERY_LIFE).await
    }

    /// Queries the UPS for info about its manufacturer, model name and version.
    pub async fn query_ups_info(&mut self) -> Result<cplus::UPSInformation> {
        self.processed_query(cplus::CMD_UPS_INFORMATION).await
    }

    /// Queries the UPS for info about its rated output voltage, current, frequency and battery voltage.
    pub async fn query_ups_rating(&mut self) -> Result<cplus::UPSRating> {
        self.processed_query(cplus::CMD_RATING_INFORMATION).await
    }
}