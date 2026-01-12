use crate::{CurvePoint, Error};

/// 最大曲线点数
pub const MAX_CURVE_POINTS: usize = 32;

/// 电压曲线
#[derive(Debug, Clone, Copy)]
pub struct Curve {
    points: [CurvePoint; MAX_CURVE_POINTS],
    len: usize,
    min_voltage: f32,
    max_voltage: f32,
}

impl Curve {
    /// 创建空曲线
    pub const fn empty() -> Self {
        Self {
            points: [CurvePoint::new(0.0, 0.0); MAX_CURVE_POINTS],
            len: 0,
            min_voltage: 0.0,
            max_voltage: 0.0,
        }
    }

    /// 创建曲线
    pub const fn new(points: &[CurvePoint]) -> Self {
        let mut curve = Self::empty();
        let mut i = 0;
    
        let mut min = 0.0_f32;
        let mut max = 0.0_f32;
    
        while i < points.len() && i < MAX_CURVE_POINTS {
            let p = points[i];
            curve.points[i] = p;
    
            if i == 0 {
                min = p.voltage;
                max = p.voltage;
            } else {
                if p.voltage < min { min = p.voltage; }
                if p.voltage > max { max = p.voltage; }
            }
    
            i += 1;
        }
    
        curve.len = i;
        if i > 0 {
            curve.min_voltage = min;
            curve.max_voltage = max;
        }
        curve
    }
    

    /// 从电压计算SOC（线性插值）
    pub fn voltage_to_soc(&self, voltage: f32) -> Result<f32, Error> {
        if self.len < 2 {
            return Err(Error::InvalidCurve);
        }

        // 边界检查
        if voltage >= self.max_voltage {
            return Ok(self.points[self.len - 1].soc);
        }
        if voltage <= self.min_voltage {
            return Ok(self.points[0].soc);
        }

        // 线性查找
        for i in 1..self.len {
            let prev = self.points[i - 1];
            let curr = self.points[i];

            if voltage >= prev.voltage && voltage <= curr.voltage {
                let ratio = (voltage - prev.voltage) / (curr.voltage - prev.voltage);
                let soc = prev.soc + ratio * (curr.soc - prev.soc);
                return Ok(soc);
            }
        }

        Err(Error::NumericalError)
    }

    /// 获取电压范围
    pub const fn voltage_range(&self) -> (f32, f32) {
        (self.min_voltage, self.max_voltage)
    }

    /// 获取曲线点数
    pub const fn len(&self) -> usize {
        self.len
    }

    /// 曲线是否为空
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// 预定义电池曲线
pub mod default_curves {
    use super::*;

    /// 锂聚合物电池曲线（电压从低到高排序）
    pub const LIPO: Curve = Curve::new(&[
        CurvePoint::new(3.20, 0.0), // 最低电压
        CurvePoint::new(3.30, 5.0),
        CurvePoint::new(3.40, 10.0),
        CurvePoint::new(3.50, 20.0),
        CurvePoint::new(3.60, 30.0),
        CurvePoint::new(3.70, 50.0), // 典型电压
        CurvePoint::new(3.80, 70.0),
        CurvePoint::new(3.90, 85.0),
        CurvePoint::new(4.00, 95.0),
        CurvePoint::new(4.20, 100.0), // 最高电压
    ]);

    /// 磷酸铁锂电池曲线（电压从低到高）
    pub const LIFEPO4: Curve = Curve::new(&[
        CurvePoint::new(2.50, 0.0),
        CurvePoint::new(2.80, 15.0),
        CurvePoint::new(3.00, 35.0),
        CurvePoint::new(3.10, 45.0),
        CurvePoint::new(3.20, 55.0),
        CurvePoint::new(3.30, 65.0), // 3.3V -> 65%
        CurvePoint::new(3.40, 75.0),
        CurvePoint::new(3.50, 85.0),
        CurvePoint::new(3.60, 95.0),
        CurvePoint::new(3.65, 100.0),
    ]);

    /// 锂离子电池曲线（电压从低到高）
    pub const LIION: Curve = Curve::new(&[
        CurvePoint::new(2.50, 0.0),
        CurvePoint::new(3.00, 30.0),
        CurvePoint::new(3.30, 50.0),
        CurvePoint::new(3.50, 65.0),
        CurvePoint::new(3.60, 70.0),
        CurvePoint::new(3.70, 75.0),
        CurvePoint::new(3.80, 80.0),
        CurvePoint::new(3.90, 85.0),
        CurvePoint::new(4.00, 90.0),
        CurvePoint::new(4.10, 95.0),
        CurvePoint::new(4.20, 100.0),
    ]);

    /// 1S 锂聚合物电池“保守满电/保守亏电”曲线（电压从低到高排序）
    ///
    /// - Vbat <= 3.40V：视为 0%，并应触发低电关机（保护电池）
    /// - Vbat >= 4.10V：视为 100%（不追求 4.20V 才满电）
    pub const LIPO410_FULL340_CUTOFF: Curve = Curve::new(&[
        CurvePoint::new(3.40, 0.0),    // 关机阈值=0%
        CurvePoint::new(3.50, 8.0),
        CurvePoint::new(3.60, 18.0),
        CurvePoint::new(3.70, 40.0), 
        CurvePoint::new(3.75, 52.0),
        CurvePoint::new(3.80, 65.0),
        CurvePoint::new(3.85, 78.0),
        CurvePoint::new(3.90, 88.0),
        CurvePoint::new(3.95, 94.0),
        CurvePoint::new(4.00, 97.0),
        CurvePoint::new(4.05, 99.0),
        CurvePoint::new(4.10, 100.0),  // 满电阈值
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curve_basic() {
        let curve = Curve::new(&[CurvePoint::new(3.0, 0.0), CurvePoint::new(4.0, 100.0)]);

        assert_eq!(curve.voltage_to_soc(3.0).unwrap(), 0.0);
        assert_eq!(curve.voltage_to_soc(4.0).unwrap(), 100.0);
        assert_eq!(curve.voltage_to_soc(3.5).unwrap(), 50.0);
    }

    #[test]
    fn test_curve_boundaries() {
        let curve = Curve::new(&[
            CurvePoint::new(3.0, 0.0),
            CurvePoint::new(3.5, 50.0),
            CurvePoint::new(4.0, 100.0),
        ]);

        // 测试边界
        assert_eq!(curve.voltage_to_soc(2.9).unwrap(), 0.0); // 低于最小值
        assert_eq!(curve.voltage_to_soc(4.1).unwrap(), 100.0); // 高于最大值

        // 测试中间值
        assert_eq!(curve.voltage_to_soc(3.25).unwrap(), 25.0);
        assert_eq!(curve.voltage_to_soc(3.75).unwrap(), 75.0);
    }

    #[test]
    fn test_curve_invalid() {
        let curve = Curve::new(&[CurvePoint::new(3.0, 0.0)]);

        // 只有一个点的曲线应该出错
        assert!(curve.voltage_to_soc(3.5).is_err());
    }
}
