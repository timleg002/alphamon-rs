use alphamon_rs::device::cplus::{self, CPlusInterface as _};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut iface = cplus::CPlusSerialInterface::connect("COM4")?; // Specify your port path

    let status = iface.query_ups_status()?;

    println!("{status:#?}");

    Ok(())
}