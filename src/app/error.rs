const EXIT_FAIL_UNDER: i32 = 1;
const EXIT_USAGE: i32 = 2;

#[derive(Debug)]
pub struct AppError {
    pub message: String,
    pub exit_code: i32,
}

impl AppError {
    pub fn new(message: impl Into<String>, exit_code: i32) -> Self {
        Self {
            message: message.into(),
            exit_code,
        }
    }

    pub fn usage(message: impl Into<String>) -> Self {
        Self::new(message, EXIT_USAGE)
    }

    pub fn fail_under(percent: f64, threshold: f64) -> Self {
        Self::new(
            format!(
                "Coverage {:.2}% is below --fail-under {:.2}%",
                percent, threshold
            ),
            EXIT_FAIL_UNDER,
        )
    }
}
