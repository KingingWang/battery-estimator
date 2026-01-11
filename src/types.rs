//! 类型定义

/// 电池化学类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BatteryChemistry {
    /// 锂聚合物电池 (4.2V 满充, 3.2V 截止)
    LiPo,
    /// 磷酸铁锂电池 (3.65V 满充, 3.0V 截止)
    LiFePO4,
    /// 锂离子电池 (4.2V 满充, 3.3V 截止)
    LiIon,
}

/// 电压-SOC曲线点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurvePoint {
    /// 电压 (伏特)
    pub voltage: f32,
    /// 电量百分比 (0-100)
    pub soc: f32,
}

impl CurvePoint {
    /// 创建新的曲线点
    pub const fn new(voltage: f32, soc: f32) -> Self {
        Self { voltage, soc }
    }
}

impl From<(f32, f32)> for CurvePoint {
    fn from((voltage, soc): (f32, f32)) -> Self {
        Self::new(voltage, soc)
    }
}

