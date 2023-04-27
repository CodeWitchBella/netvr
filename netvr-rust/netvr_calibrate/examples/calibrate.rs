use anyhow::anyhow;
use netvr_calibrate::CalibrationInput;

fn main() -> anyhow::Result<()> {
    let fname = std::env::args()
        .nth(1)
        .ok_or(anyhow!("Missing file name"))?;
    println!("Reading calibration data from file: {}", fname);
    let data = std::fs::read_to_string(fname)?;
    let calibration: CalibrationInput = serde_json::from_str(&data)?;
    let start = chrono::Utc::now();
    let result = netvr_calibrate::calibrate(&calibration);
    let elapsed = chrono::Utc::now() - start;
    println!("Calibration result: {:?}", result);
    println!("Calibration compute took: {}ms", elapsed.num_milliseconds(),);
    Ok(())
}
