#![warn(clippy::missing_safety_doc)]
pub unsafe fn unchecked() {}
pub fn repaired() -> i32 { let mut number = 1; number }
