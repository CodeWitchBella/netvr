mod calibrate;
mod input;

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
type NonWeb = ();

#[cfg(any(NonWeb))]
pub use calibrate::*;
#[cfg(any(NonWeb))]
pub use input::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[derive(serde::Serialize)]
pub struct CalibrationResultCompat {
    pub translate: netvr_data::Vec3,
    pub rotate: netvr_data::Vec3,
    pub rotateq: netvr_data::Quaternion,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn compute(samples: &str) -> String {
    let samples: input::CalibrationInput = match serde_json::from_str(samples) {
        Ok(samples) => samples,
        Err(e) => {
            return format!("{{\"error\": \"{}\"}}", e);
        }
    };
    let result = match calibrate::calibrate(&samples) {
        Ok(result) => result,
        Err(e) => {
            return format!("{{\"error\": \"{}\"}}", e);
        }
    };
    match serde_json::to_string(&CalibrationResultCompat {
        translate: result.translation,
        rotate: {
            let q = result.rotation.clone();
            let (x, y, z) = nalgebra::UnitQuaternion::new_unchecked(
                nalgebra::Quaternion::new(q.w, q.x, q.y, q.z).normalize(),
            )
            .euler_angles();
            netvr_data::Vec3 { x, y, z }
        },
        rotateq: result.rotation,
    }) {
        Ok(result) => result,
        Err(e) => {
            return format!("{{\"error\": \"{}\"}}", e);
        }
    }
}
