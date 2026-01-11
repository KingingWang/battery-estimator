use core::fmt;

/// 电池估算器库中可能出现的错误类型。
///
/// 这个枚举包含了所有可能由电池估算器操作引发的错误。
/// 使用 `Result<T, Error>` 作为函数的返回类型来提供错误处理。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    /// 电压值超出有效范围
    VoltageOutOfRange,
    /// 提供的曲线数据无效
    InvalidCurve,
    /// 数值计算错误
    NumericalError,
    /// 温度值无效
    InvalidTemperature,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::VoltageOutOfRange => write!(f, "Voltage out of valid range"),
            Error::InvalidCurve => write!(f, "Invalid voltage curve"),
            Error::NumericalError => write!(f, "Numerical error in calculation"),
            Error::InvalidTemperature => write!(f, "Invalid temperature"),
        }
    }
}
